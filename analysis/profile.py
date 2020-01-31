import pdb

import os
import subprocess
import re
import toml
import numpy as np
import matplotlib.pyplot as plt
import matplotlib as matplotlib
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
        projection = configuration["clustered_light_shading"]["projection"]
        if projection == "Orthographic":
            size = configuration["clustered_light_shading"]["orthographic_sides"]
            return ClusteringProjectionOrthographic(size["x"], size["y"], size["z"])
        if projection == "Perspective":
            size = configuration["clustered_light_shading"]["perspective_pixels"]
            return ClusteringProjectionPerspective(size["x"], size["y"])

class ClusteringProjectionOrthographic(ClusteringProjection):
    def __init__(self, x, y, z):
        super().__init__()
        self.x = x
        self.y = y
        self.z = z

    def short_name(self):
        if self.x == self.y and self.y == self.z:
            return "ortho {}".format(self.x)
        else:
            return "ortho {}x{}x{}".format(self.x, self.y, self.z)

    def __as_tuple(self):
        return (self.x, self.y, self.z)

    def __eq__(self, other):
        return isinstance(other, type(self)) and self.__as_tuple() == other.__as_tuple()

    def __hash__(self):
        return hash(self.__as_tuple())

class ClusteringProjectionPerspective(ClusteringProjection):
    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    def short_name(self):
        if self.x == self.y:
            return "persp {}".format(self.x)
        else:
            return "persp {}x{}".format(self.x, self.y)

    def __as_tuple(self):
        return (self.x, self.y)

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
    ClusteringProjectionOrthographic(4.0, 4.0, 4.0),
    ClusteringProjectionPerspective(64, 64),
]

sample_tuples = [
    ("Total (ms)", "/frame"),
    ("Clustering (ms)", "/frame/cluster"),
    ("Shading (ms)", "/frame/basic"),
]

def generate_tune_plots(profiles):
    techniques = [
        ("ortho", "Orthographic", lambda profile: profile.configuration["clustered_light_shading"]["orthographic_sides"]["x"]),
        ("persp", "Perspective", lambda profile: profile.configuration["clustered_light_shading"]["perspective_pixels"]["x"]),
    ]
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        for (technique_short_name, technique_enum, technique_sort) in techniques:
            figsize = (thesis.textwidth, thesis.textwidth *2/3)
            fig, axes = plt.subplots(len(sample_tuples), len(lightings), sharex = 'col', squeeze=False, figsize=figsize, dpi = thesis.dpi,
                gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 1.5, 0.3)
            )

            for row, (sample_label, sample_path) in enumerate(sample_tuples):
                for col, lighting in enumerate(lightings):
                    ax = axes[row, col]

                    if row == 0:
                        ax.set_title("{} lights (r1 = {:.2f})".format(
                            lighting.count,
                            lighting.attenuation.r1
                        ))

                    if row + 1 == len(sample_tuples):
                        ax.set_xlabel("frame")
                    else:
                        ax.tick_params(axis='x',bottom=False)

                    if col == 0:
                        ax.set_ylabel("{}".format(sample_label))

                    profile_selection = sorted([
                        profile for profile in profiles
                        if profile.configuration["global"]["scene_path"] == scene_path
                        and lighting == Lighting.from_configuration(profile.configuration)
                        and profile.configuration["clustered_light_shading"]["projection"] == technique_enum
                    ], key = technique_sort)

                    color_palette = plt.cm.get_cmap('Blues', len(profile_selection)+1)

                    for color_index, profile in enumerate(profile_selection):
                        samples = profile.samples.min_gpu_samples_by_name(sample_path)
                        projection = ClusteringProjection.from_configuration(profile.configuration)
                        ax.plot(samples, color=color_palette(color_index+1), label=projection.short_name())
                        ax.set_xlim(0, len(samples) - 1)

                    if row == 0 and col == 0:
                        ax.legend(loc = 'upper left')

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/tune_{}_{}.pdf'.format(technique_short_name, scene_name), format='pdf')

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
        figsize = (thesis.textwidth, thesis.textwidth * 2/3 * 2/3)
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

def generate_ortho_vs_persp_plots(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    why_dont_i_have_scopes000 = [
        ("Total Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame")),
        ("Shading Time (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/basic")),
        ("Shading Operations", lambda samples: samples.sum_shading_operations()),
        ("Clustering (ms)", lambda samples: samples.min_gpu_samples_by_name("/frame/cluster")),
        ("Visible Clusters", lambda samples: samples.sum_visible_clusters()),
        ("Light Indices", lambda samples: samples.sum_light_indices()),
    ]

    for (scene_name, scene_path) in scenes:
        row_count = len(why_dont_i_have_scopes000)
        col_count = len(lightings)
        figsize = (thesis.textwidth, thesis.textwidth * row_count/col_count * 2/3)
        fig, axes = plt.subplots(
            row_count, col_count, sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for row, (sample_label, sample_func) in enumerate(why_dont_i_have_scopes000):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == row_count:
                    ax.set_xlabel("frame")
                else:
                    ax.tick_params(axis='x',bottom=False)

                if col == 0:
                    ax.set_ylabel("{}".format(sample_label))
                else:
                    ax.tick_params(axis='y',left=False)

                for projection in tuned_projections:
                    profile = single([profile for profile in profiles
                        if profile.configuration["global"]["scene_path"] == scene_path
                        and lighting == Lighting.from_configuration(profile.configuration)
                        and projection == ClusteringProjection.from_configuration(profile.configuration)
                    ])

                    samples = sample_func(profile.samples)
                    ax.plot(samples, label=projection.short_name())
                    ax.set_xlim(0, len(samples) - 1)

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/ortho_vs_persp_{}.pdf'.format(scene_name), format='pdf')

def generate_indi_vs_encl(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    sample_labels = [
        "Shading Operations",
        "Visible Clusters",
        "Light Indices",
    ]

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(sample_tuples), len(lightings), sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for row, sample_label in enumerate(sample_labels):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
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

                for color_base, projection in enumerate(tuned_projections):
                    for color_offset, (linestyle, grouping) in enumerate([ ("-", "Individual"), (":", "Enclosed") ]):
                        profile = single([
                            profile for profile in profiles
                                if scene_path == profile.configuration["global"]["scene_path"]
                                and lighting == Lighting.from_configuration(profile.configuration)
                                and projection == ClusteringProjection.from_configuration(profile.configuration)
                                and grouping == profile.configuration["clustered_light_shading"]["grouping"]
                        ])

                        if row == 0:
                            samples = profile.samples.sum_shading_operations()
                        if row == 1:
                            samples = profile.samples.sum_visible_clusters()
                        if row == 2:
                            samples = profile.samples.sum_light_indices()

                        ax.plot(samples, color=color_palette(color_base * 4 + color_offset), linestyle=linestyle, label="{} {}".format(grouping, projection.short_name()))
                        ax.set_xlim(0, len(samples) - 1)

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/stereo_atemporal_{}.pdf'.format(scene_name), format='pdf')

def generate_heatmap(profiles):
    lightings = sorted({ Lighting.from_configuration(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    sample_labels = [
        "Shading Operations",
        "Visible Clusters",
        "Light Indices",
    ]

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(1, 3*len(tuned_projections), sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.2, 0.2)
        )

        samples_array = [
            ("Frags/Clus", "log$_2$(count)", lambda samples: samples.cluster_buffers[:,0,256:512]),
            ("Lights/Clus", "count/8", lambda samples: samples.cluster_buffers[:,0,512:768]),
            ("Lights/Frag", "count/8", lambda samples: samples.cluster_buffers[:,0,768:1024]),
        ]

        for si, (label, xlabel, sample_func) in enumerate(samples_array):
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

            ortho_samples = sample_func(ortho_profile.samples)
            persp_samples = sample_func(persp_profile.samples)

            vmin = np.min(np.vstack([ortho_samples, persp_samples]))
            vmax = np.max(np.vstack([ortho_samples, persp_samples]))

            ax = axes[0, si*2]
            im = ax.imshow(ortho_samples, aspect=1/4, interpolation='nearest', vmin=vmin, vmax=vmax)

            ax = axes[0, si*2 + 1]
            im = ax.imshow(persp_samples, aspect=1/4, interpolation='nearest', vmin=vmin, vmax=vmax)

            if si == 0:
                ax.set_ylabel("frame")

            ax.set_xlabel(xlabel)
            ax.set_title(label)

profile_dir_regex = re.compile(r"^(suntem|bistro)_[ \d]{7}_(ortho|persp)_[ \d]{4}$");
profiles_0 = load_profiles("../profiling", profile_dir_regex);

# generate_tune_plots(profiles_0)
# generate_stackplots(profiles_0)
# generate_ortho_vs_persp_plots(profiles_0)
generate_heatmap(profiles_0)

stereo_profile_dir_regex = re.compile(r"^stereo_(suntem|bistro)_\d{7}_(indi|encl)_(ortho|persp)_\d{4}$")
stereo_profiles_0 = load_profiles("../profiling", stereo_profile_dir_regex)

# generate_indi_vs_encl(stereo_profiles_0)

plt.show()
