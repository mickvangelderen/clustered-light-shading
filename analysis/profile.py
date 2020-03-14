import pdb

import os
import subprocess
import re
import toml
import numpy as np
import matplotlib.pyplot as plt
import matplotlib as matplotlib
from mpl_toolkits.axes_grid1 import AxesGrid
from profile_samples import ProfileSamples
import thesis

# tab10modern = [
#     '#4e79a7',
#     '#f28e2b',
#     '#e15759',
#     '#76b7b2',
#     '#59a14e',
#     '#edc949',
#     '#b07aa2',
#     '#ff9da7',
#     '#9c755f',
#     '#bab0ac',
# ]

# plt.register_cmap(cmap=matplotlib.colors.ListedColormap(tab10modern, name='tab10modern'))
# plt.set_cmap('tab10modern')

def group_by(values, key):
    res = {};
    for v in values:
        k = key(v)
        if k in res:
            res[k].append(v)
        else:
            res[k] = [v]
    return res

def single(values):
    if 1 != len(values):
        print(values)
    assert 1 == len(values)
    return values[0]

class Attenuation:
    def __init__(self, configuration):
        self.i = configuration["i"]
        self.i0 = configuration["i0"]
        self.r0 = configuration["r0"]
        self.r1 = np.sqrt(self.i / self.i0)

    def __eq__(self, other):
        return isinstance(other, type(self)) and (self.i, self.i0, self.r0) == (other.i, other.i0, other.r0)

    def __hash__(self):
        return hash((self.i, self.i0, self.r0))

class Lighting:
    def __init__(self, count, attenuation):
        self.count = count
        self.attenuation = attenuation

    def from_configuration(configuration):
        return Lighting(
            count = configuration["rain"]["max_count"],
            attenuation = Attenuation(configuration["light"]["attenuation"]),
        )

    def short_name(self):
        return "{} lights ($R_1$ = {:.2f})".format(self.count, self.attenuation.r1)

    def __eq__(self, other):
        return isinstance(other, type(self)) and (self.count, self.attenuation) == (other.count, other.attenuation)

    def __hash__(self):
        return hash((self.count, self.attenuation))

class ClusteringProjection:
    def from_configuration(configuration):
        cls = configuration["clustered_light_shading"]
        projection = cls["projection"]
        if projection == "Orthographic":
            size = cls["orthographic_sides"]
            return ClusteringProjectionOrthographic(size["x"], size["y"], size["z"])
        if projection == "Perspective":
            size = cls["perspective_pixels"]
            displacement = cls["perspective_displacement"]
            return ClusteringProjectionPerspective(size["x"], size["y"], displacement)

class ClusteringProjectionOrthographic(ClusteringProjection):
    def __init__(self, x, y, z):
        super().__init__()
        self.x = x
        self.y = y
        self.z = z

    def short_name(self):
        if self.x == self.y and self.y == self.z:
            return r"\texttt{{Ortho}} {}".format(self.x)
        else:
            return r"\texttt{{Ortho}} {}x{}x{}".format(self.x, self.y, self.z)

    def color_name(self):
        return "Blues"

    def __as_tuple(self):
        return (self.x, self.y, self.z)

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.__as_tuple() == other.__as_tuple()

    def __hash__(self):
        return hash(self.__as_tuple())

class ClusteringProjectionPerspective(ClusteringProjection):
    def __init__(self, x, y, displacement):
        super().__init__()
        self.x = x
        self.y = y
        self.displacement = displacement

    def short_name(self):
        dimensions_string = "{}".format(self.x) if self.x == self.y else "{}x{}".format(self.x, self.y)
        return r"\texttt{{Persp}} {}".format(dimensions_string) if self.displacement == 0.0 else r"\texttt{{Displ}} {} {:.0f}".format(dimensions_string, self.displacement)

    def color_name(self):
        return "Oranges" if self.displacement == 0.0 else "Greens"

    def __as_tuple(self):
        return (self.x, self.y, self.displacement)

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.__as_tuple() == other.__as_tuple()

    def __hash__(self):
        return hash(self.__as_tuple())

class Profile:
    def __init__(self, directory):
        self.directory = directory
        self.configuration = toml.load(os.path.join(directory, "configuration.toml"))
        self.samples = ProfileSamples(directory)

def load_profiles(profiling_directory, regex):
    with os.scandir(profiling_directory) as it:
        directories = [os.path.join(profiling_directory, entry.name) for entry in it if entry.is_dir() and regex.match(entry.name)]

    return [Profile(directory) for directory in directories]

def gridspec_box(l, r, b, t, w, h, ws, hs):
    return {
        "left": l/w,
        "right": (w - r)/w,
        "bottom": b/h,
        "top": (h - t)/h,
        "wspace": ws/w,
        "hspace": hs/h,
    }

scenes = [
    ("bistro", "bistro/Bistro_Exterior.bin"),
    ("suntem", "sun_temple/SunTemple.bin"),
]

tuned_projections = [
    ClusteringProjectionPerspective(64, 64, 0.0),
    ClusteringProjectionOrthographic(4.0, 4.0, 4.0),
    ClusteringProjectionPerspective(64, 64, 32.0),
]

sample_tuples = [
    ("Total (ms)", "/frame"),
    ("Clustering (ms)", "/frame/cluster"),
    ("Shading (ms)", "/frame/basic"),
]

def generate_tune_plots(profiles):
    projection_map = {
        "persp": [ClusteringProjectionPerspective(side, side, 0.0) for side in [16, 32, 64, 96, 128]],
        "ortho": [ClusteringProjectionOrthographic(side, side, side) for side in [1.0, 2.0, 4.0, 8.0, 16.0]],
        "displ": [ClusteringProjectionPerspective(64, 64, displacement) for displacement in [1.0, 4.0, 32.0, 256.0]],
        "tuned": [
            ClusteringProjectionPerspective(64, 64, 0.0),
            ClusteringProjectionOrthographic(4.0, 4.0, 4.0),
            ClusteringProjectionPerspective(64, 64, 32.0),
        ]
    }

    sample_label_funcs = [
        ("Visible Clusters", lambda samples: samples.sum_visible_clusters()),
        ("Clustering (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/cluster")),
        ("Lighting Computations", lambda samples: samples.sum_shading_operations()),
        ("Shading Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/basic")),
        ("Total Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame")),
        # ("Total Light Indices", lambda samples: samples.sum_light_indices()),
    ]

    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        for projection_group, projection_list in projection_map.items():
            row_count = len(sample_label_funcs)
            col_count = len(lightings)
            figsize = (thesis.textwidth, thesis.textwidth * row_count/col_count * 2/3)
            fig, axes = plt.subplots(row_count, col_count, sharex = 'col', squeeze=False, figsize=figsize, dpi = thesis.dpi,
                gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 1.5, 0.3)
            )

            for row, (sample_label, sample_func) in enumerate(sample_label_funcs):
                for col, lighting in enumerate(lightings):
                    ax = axes[row, col]

                    if row == 0:
                        ax.set_title("{} lights ($R_1 = {:.2f}$)".format(
                            lighting.count,
                            lighting.attenuation.r1
                        ))

                    if row + 1 == len(sample_tuples):
                        ax.set_xlabel("frame")
                    else:
                        ax.tick_params(axis='x',bottom=False)

                    if col == 0:
                        ax.set_ylabel("{}".format(sample_label))

                    cmap_names, cmap_counts = np.unique([t.color_name() for t in projection_list], return_counts=True)
                    cmaps = {
                        n: plt.cm.get_cmap(n, c + 2) for n, c in zip(cmap_names, cmap_counts)
                    }
                    cmap_counts = { n: 0 for n in cmap_names }

                    for color_index, projection in enumerate(projection_list):
                        profile = single([profile for profile in profiles
                            if profile.configuration["global"]["scene_path"] == scene_path
                            and projection == ClusteringProjection.from_configuration(profile.configuration)
                            and lighting == Lighting.from_configuration(profile.configuration)
                        ])
                        samples = sample_func(profile.samples)

                        color_name = projection.color_name()
                        cmap_counts[color_name] += 1
                        color = cmaps[color_name](cmap_counts[color_name])

                        ax.plot(samples, color=color, label=projection.short_name())
                        ax.set_xlim(0, len(samples) - 1)

                    if row == 0 and col == 0:
                        ax.legend(loc = 'upper left')

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/tune_{}_{}.pdf'.format(projection_group, scene_name), format='pdf')

        for lighting in lightings:
            for (scene_name, frames) in [
                ("bistro", [300, 580]),
                ("suntem", [250, 680]),
            ]:
                for frame in frames:
                    i = "../profiling/{}_{:07}_ortho_0400/frames/{}.jpg".format(scene_name, lighting.count, frame)
                    o = "../../thesis/media/{}_{:07}_ortho_0400_{}.jpg".format(scene_name, lighting.count, frame)
                    subprocess.run(["cp", i, o])

def generate_stackplots(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(tuned_projections), len(lightings), sharex = 'col', sharey = 'all', squeeze=False, figsize = figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for col, lighting in enumerate(lightings):
            for row, projection in enumerate(tuned_projections):
                ax = axes[row, col]

                profile = single([profile for profile in profiles
                    if profile.configuration["global"]["scene_path"] == scene_path
                    and projection == ClusteringProjection.from_configuration(profile.configuration)
                    and lighting == Lighting.from_configuration(profile.configuration)])

                frame_samples = profile.samples.min_gpu_samples_by_name("/frame")
                cluster_samples = profile.samples.min_gpu_samples_by_name("/frame/cluster")
                basic_samples = profile.samples.min_gpu_samples_by_name("/frame/basic")
                misc_samples = frame_samples - cluster_samples - basic_samples
                stacked_samples = np.vstack([basic_samples, cluster_samples, misc_samples])
                stacked_labels = ["Shading", "Clustering", "Miscellaneous"]

                x0 = 0
                x1 = stacked_samples.shape[1]

                ax.stackplot(np.arange(x0, x1), stacked_samples, labels=stacked_labels)
                ax.set_xlim(x0, x1 - 1)

                if col == 0 and row == 0:
                    ax.legend(loc = 'upper left')

                if col == 0:
                    ax.set_ylabel("{}\nGPU time (ms)".format(projection.short_name()))
                else:
                    ax.tick_params(axis='y',left=False)

                if row == 0:
                    ax.set_title(lighting.short_name())

                if row + 1 == np.shape(axes)[0]:
                    ax.set_xlabel("frame")
                else:
                    ax.tick_params(axis='x',bottom=False)

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/stack_{}.pdf'.format(scene_name), format='pdf')

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(tuned_projections), len(lightings), sharex = 'col', sharey = 'all', squeeze=False, figsize = figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for col, lighting in enumerate(lightings):
            for row, projection in enumerate(tuned_projections):
                ax = axes[row, col]

                profile = single([profile for profile in profiles
                    if profile.configuration["global"]["scene_path"] == scene_path
                    and projection == ClusteringProjection.from_configuration(profile.configuration)
                    and lighting == Lighting.from_configuration(profile.configuration)])

                local_sample_tuples = [
                    ('Visibility', '/frame/cluster/camera'),
                    ('Count Lights', '/frame/cluster/count_lights'),
                    ('Assign Lights', '/frame/cluster/assign_lights'),
                ]

                cluster_samples = profile.samples.min_gpu_samples_by_name("/frame/cluster")
                other_samples = [profile.samples.min_gpu_samples_by_name(sample_path) for sample_label, sample_path in local_sample_tuples]
                misc_samples = cluster_samples - np.sum(np.vstack(other_samples), axis=0)
                stacked_samples = np.vstack(other_samples + [misc_samples])
                stacked_labels = [sample_label for sample_label, sample_path in local_sample_tuples] + ['Miscellaneous']

                x0 = 0
                x1 = stacked_samples.shape[1]

                ax.stackplot(np.arange(x0, x1), stacked_samples, labels=stacked_labels)
                ax.set_xlim(x0, x1 - 1)

                if col == 0 and row == 0:
                    ax.legend(loc = 'upper left')

                if col == 0:
                    ax.set_ylabel("{}\nGPU time (ms)".format(projection.short_name()))
                else:
                    ax.tick_params(axis='y',left=False)

                if row == 0:
                    ax.set_title(lighting.short_name())

                if row + 1 == np.shape(axes)[0]:
                    ax.set_xlabel("frame")
                else:
                    ax.tick_params(axis='x',bottom=False)

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/stack_clustering_{}.pdf'.format(scene_name), format='pdf')

def generate_indi_vs_encl(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    sample_label_funcs = [
        ("Visible Clusters", lambda samples: samples.sum_visible_clusters()),
        ("Clustering (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/cluster")),
        ("Lighting Computations", lambda samples: samples.sum_shading_operations()),
        ("Shading Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/basic")),
        ("Total Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame")),
        # ("Total Light Indices", lambda samples: samples.sum_light_indices()),
    ]

    for (scene_name, scene_path) in scenes:
        col_count = len(lightings)
        row_count = len(sample_label_funcs)
        figsize = (thesis.textwidth, thesis.textwidth * row_count/col_count * 2/3)
        fig, axes = plt.subplots(row_count, col_count, sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for row, (sample_label, sample_func) in enumerate(sample_label_funcs):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights ($R_1 = {:.2f}$)".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == np.shape(axes)[0]:
                    ax.set_xlabel("frame")
                else:
                    ax.tick_params(axis='x',bottom=False)

                if col == 0:
                    ax.set_ylabel(sample_label)
                else:
                    ax.tick_params(axis='y',left=False)

                color_palette = plt.get_cmap("tab20c")

                for color_base, projection in enumerate(tuned_projections[:2]):
                    for color_offset, (linestyle, grouping) in enumerate([ ("-", "Individual"), (":", "Enclosed") ]):
                        profile = single([
                            profile for profile in profiles
                                if scene_path == profile.configuration["global"]["scene_path"]
                                and lighting == Lighting.from_configuration(profile.configuration)
                                and projection == ClusteringProjection.from_configuration(profile.configuration)
                                and grouping == profile.configuration["clustered_light_shading"]["grouping"]
                        ])

                        samples = sample_func(profile.samples)

                        ax.plot(samples, color=color_palette(color_base * 4 + color_offset), linestyle=linestyle, label="{} {}".format(grouping, projection.short_name()))
                        ax.set_xlim(0, len(samples) - 1)

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/stereo_atemporal_{}.pdf'.format(scene_name), format='pdf')

def generate_heatmap(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    samples_array = [
        ("fragments_per_cluster", "Fragments per Cluster", "Cluster\nCount", "Total\nCluster Count", "Binned by\n" + r"$\left\lfloor 8\log_2(\textrm{Frag. Cnt.})\right\rfloor$", lambda samples: samples.cluster_buffers[:,0,257:512], 4),
        ("lights_per_cluster", "Lights per Cluster", "Cluster\nCount", "Total\nCluster Count", "Binned by\nLight Count", lambda samples: samples.cluster_buffers[:,0,512:768], 4),
        ("lights_per_fragment", "Lights per Fragment", "Fragment\nCount", "Total\nFragment Count", "Binned by\nLight Count", lambda samples: samples.cluster_buffers[:,0,768:1024], 20),
    ]

    # https://jdhao.github.io/2017/06/11/mpl_multiplot_one_colorbar/

    for (scene_name, scene_path) in scenes:
        for path_suffix, suptitle, count_label, sum_label, bin_label, hist_sample_func, base in samples_array:
            row_count = 2
            col_count = 3
            figsize = (thesis.textwidth, 2/5 * thesis.textwidth)
            fig, axes = plt.subplots(row_count, col_count, sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
                gridspec_kw = gridspec_box(0.8, 0.8, 0.4, 0.2, figsize[0], figsize[1], 0.0, 0.0)
            )
            # fig.suptitle(suptitle)

            ortho_profile = single([
                profile for profile in profiles
                    if scene_path == profile.configuration["global"]["scene_path"]
                    and lightings[2] == Lighting.from_configuration(profile.configuration)
                    and tuned_projections[0] == ClusteringProjection.from_configuration(profile.configuration)
            ])

            persp_profile = single([
                profile for profile in profiles
                    if scene_path == profile.configuration["global"]["scene_path"]
                    and lightings[2] == Lighting.from_configuration(profile.configuration)
                    and tuned_projections[1] == ClusteringProjection.from_configuration(profile.configuration)
            ])

            displ_profile = single([
                profile for profile in profiles
                    if scene_path == profile.configuration["global"]["scene_path"]
                    and lightings[2] == Lighting.from_configuration(profile.configuration)
                    and tuned_projections[2] == ClusteringProjection.from_configuration(profile.configuration)
            ])

            # Divide each frame by the sum.
            ortho_samp = np.transpose(hist_sample_func(ortho_profile.samples))
            ortho_hist = np.floor(np.log2(ortho_samp * 2 + 1))
            ortho_fsum = np.sum(ortho_samp, axis = 0)
            # ortho_dist = ortho_hist/ortho_fsum

            persp_samp = np.transpose(hist_sample_func(persp_profile.samples))
            persp_hist = np.floor(np.log2(persp_samp * 2 + 1))
            persp_fsum = np.sum(persp_samp, axis = 0)
            # persp_dist = persp_hist/persp_fsum

            displ_samp = np.transpose(hist_sample_func(displ_profile.samples))
            displ_hist = np.floor(np.log2(displ_samp * 2 + 1))
            displ_fsum = np.sum(displ_samp, axis = 0)
            # displ_dist = displ_hist/displ_fsum

            vmin = 1
            vmax = np.max(np.vstack([ortho_samp, persp_samp, displ_samp]))

            # norm = matplotlib.colors.LogNorm(vmin=vmin, vmax=vmax)
            boundaries = [0, 1, base, base**2, base**3, base**4, base**5]
            norm = matplotlib.colors.BoundaryNorm(boundaries=boundaries, ncolors=len(boundaries) - 1)
            cmap = matplotlib.cm.get_cmap('viridis', len(boundaries) - 1)

            ax = axes[0, 0]
            ax.imshow(ortho_samp, origin='lower', interpolation='nearest', norm=norm, cmap=cmap)
            ax.set_title(tuned_projections[0].short_name())
            ax.set_xlim(0, len(ortho_fsum) - 1)
            ax.set_ylabel(bin_label)
            ax.tick_params(axis='x',bottom=False)

            ax = axes[0, 1]
            ax.imshow(persp_samp, origin='lower', interpolation='nearest', norm=norm, cmap=cmap)
            ax.set_title(tuned_projections[1].short_name())
            ax.set_xlim(0, len(persp_fsum) - 1)
            ax.tick_params(axis='x',bottom=False)
            ax.tick_params(axis='y',left=False)

            ax = axes[0, 2]
            ax.imshow(displ_samp, origin='lower', interpolation='nearest', norm=norm, cmap=cmap)
            ax.set_title(tuned_projections[2].short_name())
            ax.set_xlim(0, len(displ_fsum) - 1)
            ax.tick_params(axis='x',bottom=False)
            ax.tick_params(axis='y',left=False)

            ax = axes[1, 0]
            color_palette = plt.get_cmap(tuned_projections[0].color_name(), 3)
            ax.plot(ortho_fsum, color=color_palette(1))
            ax.set_xlabel("Frame")
            ax.set_ylabel(sum_label)

            ax = axes[1, 1]
            color_palette = plt.get_cmap(tuned_projections[1].color_name(), 3)
            ax.plot(persp_fsum, color=color_palette(1))
            ax.set_xlabel("Frame")

            ax = axes[1, 2]
            color_palette = plt.get_cmap(tuned_projections[2].color_name(), 3)
            ax.plot(displ_fsum, color=color_palette(1))
            ax.set_xlabel("Frame")
            ax.tick_params(axis='y',left=False)

            fig.align_ylabels(axes)

            colorbar_ax = fig.add_axes([0.9, 0.4/figsize[1], 0.02, 1 - 0.78/figsize[1]])
            colorbar = fig.colorbar(matplotlib.cm.ScalarMappable(norm=norm, cmap=cmap), cax=colorbar_ax, ax=axes[0, :], format='%1i')
            colorbar_ax.set_title(count_label)

            output_path = '../../thesis/media/heatmap_{}_{}.pdf'.format(scene_name, path_suffix)
            fig.savefig(output_path, format='pdf', dpi=thesis.dpi)

def generate_indi_vs_encl_pres(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    lightings = [lighting for lighting in lightings if lighting.count == 10000]

    sample_labels = [
        ("Total Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame")),
        ("Shading Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/basic")),
        ("Shading Operations", lambda samples: samples.sum_shading_operations()),
        ("Clustering (ms)", lambda samples:
                samples.min_gpu_samples_by_name("/frame/cluster"),
        ),
        ("Light Assignment (ms)", lambda samples:
         np.sum(np.vstack([
             samples.min_gpu_samples_by_name("/frame/cluster/count_lights"),
             samples.min_gpu_samples_by_name("/frame/cluster/compact_lights"),
             samples.min_gpu_samples_by_name("/frame/cluster/assign_lights")
         ]), axis=0)
        ),
        ("Visible Clusters", lambda samples: samples.sum_visible_clusters()),
        ("Light Indices", lambda samples: samples.sum_light_indices()),
    ]

    sample_groups = [
        [0],
        [1],
        [2],
        [1, 2],
        [3],
        [4],
        [5],
        [4, 5],
    ]

    for (scene_name, scene_path) in scenes:
        for figure_index, sample_group in enumerate(sample_groups):
            local_sample_labels = [sample_labels[sample_index] for sample_index in sample_group]

            row_count = len(local_sample_labels)
            col_count = len(lightings)
            figsize = (thesis.textwidth, thesis.textwidth * col_count/row_count * 2/3)
            fig, axes = plt.subplots(row_count, col_count, sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
                gridspec_kw = gridspec_box(1.5, 0.01, 1.0, 0.3, figsize[0], figsize[1], 0.0, 0.0)
            )

            for row, (sample_label, sample_func) in enumerate(local_sample_labels):
                for col, lighting in enumerate(lightings):

                    ax = axes[row, col]

                    # if row == 0:
                    #     ax.set_title("{} lights ($R_1 = {:.2f}$)".format(
                    #         lighting.count,
                    #         lighting.attenuation.r1
                    #     ))

                    if row + 1 == np.shape(axes)[0]:
                        ax.set_xlabel("frame")
                    else:
                        ax.tick_params(axis='x',bottom=False)

                    if col == 0:
                        ax.set_ylabel(sample_label)
                    else:
                        ax.tick_params(axis='y',left=False)

                    # color_palette = plt.get_cmap("tab20c")

                    for color_base, projection in enumerate(tuned_projections[1:2]):
                        for color_offset, (linestyle, grouping) in enumerate([ ("-", "Individual"), (":", "Enclosed") ]):
                            profile = single([
                                profile for profile in profiles
                                    if scene_path == profile.configuration["global"]["scene_path"]
                                    and lighting == Lighting.from_configuration(profile.configuration)
                                    and projection == ClusteringProjection.from_configuration(profile.configuration)
                                    and grouping == profile.configuration["clustered_light_shading"]["grouping"]
                            ])

                            samples = sample_func(profile.samples);

                            ax.plot(samples, label="{}".format(grouping))
                            # ax.plot(samples, color=color_palette(color_base * 4 + color_offset), linestyle=linestyle, label="{}".format(grouping))
                            ax.set_xlim(0, len(samples) - 1)

                    if row == 0 and col == 0:
                        ax.legend(loc = 'upper left')

            fig.align_ylabels(axes)

            output_path = 'stereo_atemporal_{}_{}.png'.format(scene_name, figure_index)
            fig.savefig(output_path, format='png', dpi=thesis.dpi)
            # subprocess.run(["mogrify", "-format", "jpg", output_path])

profile_dir_regex = re.compile(r"^(suntem|bistro)_\d{7}_(ortho|persp)_\d{4}(_\d+)?$");
profiles_0 = load_profiles("../profiling", profile_dir_regex);

generate_tune_plots(profiles_0)
generate_stackplots(profiles_0)
generate_heatmap(profiles_0)

stereo_profile_dir_regex = re.compile(r"^stereo_(suntem|bistro)_\d{7}_(indi|encl)_(ortho|persp)_\d{4}$")
stereo_profiles_0 = load_profiles("../profiling", stereo_profile_dir_regex)

# for name in stereo_profiles_0[0].samples.sample_names:
#     print(name)

generate_indi_vs_encl(stereo_profiles_0)
# generate_indi_vs_encl_pres(stereo_profiles_0)

# plt.show()
