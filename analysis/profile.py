import pdb

import os
import subprocess
import re
import toml
import numpy as np
import matplotlib.pyplot as plt
from profile_samples import ProfileSamples
import thesis

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
    def __init__(self, configuration):
        self.count = configuration["rain"]["max_count"]
        self.attenuation = Attenuation(configuration["light"]["attenuation"])

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

sample_tuples = [("Total", "/frame"), ("Clustering", "/frame/cluster"), ("Shading", "/frame/basic")]

def generate_tune_plots(profiles):
    techniques = [
        ("ortho", "Orthographic", lambda profile: profile.configuration["clustered_light_shading"]["orthographic_sides"]["x"]),
        ("persp", "Perspective", lambda profile: profile.configuration["clustered_light_shading"]["perspective_pixels"]["x"]),
    ]
    lightings = sorted({ Lighting(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        for (technique_short_name, technique_enum, technique_sort) in techniques:
            figsize = (thesis.textwidth, thesis.textwidth *2/3)
            fig, axes = plt.subplots(len(sample_tuples), len(lightings), sharex = 'col', squeeze=False, figsize=figsize, dpi = thesis.dpi,
                gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 1.5, 0.2)
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
                        ax.set_ylabel("{} (ms)".format(sample_label))

                    profile_selection = sorted([
                        profile for profile in profiles
                        if profile.configuration["global"]["scene_path"] == scene_path
                        and lighting == Lighting(profile.configuration)
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
    lightings = sorted({ Lighting(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(tuned_projections), len(lightings), sharex = 'col', sharey = 'all', squeeze=False, figsize = figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.0, 0.0)
        )

        for row, projection in enumerate(tuned_projections):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                profile = single([profile for profile in profiles
                    if profile.configuration["global"]["scene_path"] == scene_path
                    and projection == ClusteringProjection.from_configuration(profile.configuration)
                    and lighting == Lighting(profile.configuration)])

                frame_samples = profile.samples.min_gpu_samples_by_name("/frame")
                cluster_samples = profile.samples.min_gpu_samples_by_name("/frame/cluster")
                basic_samples = profile.samples.min_gpu_samples_by_name("/frame/basic")
                misc_samples = frame_samples - cluster_samples - basic_samples
                stacked_samples = np.vstack([basic_samples, cluster_samples, misc_samples])
                stacked_labels = ["shading", "clustering", "miscellaneous"]

                x0 = 0
                x1 = stacked_samples.shape[1]

                ax.stackplot(np.arange(x0, x1), stacked_samples, labels=stacked_labels)
                ax.set_xlim(x0, x1 - 1)

                if col == 0 and row == 0:
                    ax.legend()

                if col == 0:
                    ax.set_ylabel("{}\nGPU time (ms)".format(projection.short_name()))
                else:
                    ax.tick_params(axis='y',left=False)

                if row == 0:
                    ax.set_title(lighting.short_name())

                if row + 1 == len(tuned_projections):
                    ax.set_xlabel("frame")
                else:
                    ax.tick_params(axis='x',bottom=False)

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/stack_{}.pdf'.format(scene_name), format='pdf')

def generate_ortho_vs_persp_plots(profiles):
    lightings = sorted({ Lighting(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(sample_tuples), len(lightings), sharex = 'col', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 1.5, 0.2)
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
                    ax.set_ylabel("{} (ms)".format(sample_label))

                for projection in tuned_projections:
                    profile = single([profile for profile in profiles
                        if profile.configuration["global"]["scene_path"] == scene_path
                        and lighting == Lighting(profile.configuration)
                        and projection == ClusteringProjection.from_configuration(profile.configuration)
                    ])

                    samples = profile.samples.min_gpu_samples_by_name(sample_path)
                    ax.plot(samples, label=projection.short_name())
                    ax.set_xlim(0, len(samples) - 1)

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/ortho_vs_persp_{}.pdf'.format(scene_name), format='pdf')

def generate_stereo_plots(profiles):
    lightings = sorted({ Lighting(profile.configuration) for profile in profiles }, key = lambda x: x.count)

    sample_labels = [
        "shading operations",
        "cluster count",
        "light indices",
    ]

    for (scene_name, scene_path) in scenes:
        figsize = (thesis.textwidth, thesis.textwidth * 2/3)
        fig, axes = plt.subplots(len(sample_tuples), len(lightings), sharex = 'col', sharey = 'row', squeeze=False, figsize=figsize, dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.01, 0.5, 0.3, figsize[0], figsize[1], 0.2, 0.2)
        )

        for row, sample_label in enumerate(sample_labels):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == len(sample_labels):
                    ax.set_xlabel("frame")

                if col == 0:
                    ax.set_ylabel(sample_label)

                color_palette = plt.get_cmap("tab20c")

                for color_base, projection in enumerate(tuned_projections):
                    for color_offset, (linestyle, grouping) in enumerate([ ("-", "Individual"), (":", "Enclosed") ]):
                        profile = single([
                            profile for profile in profiles
                                if scene_path == profile.configuration["global"]["scene_path"]
                                and lighting == Lighting(profile.configuration)
                                and projection == ClusteringProjection.from_configuration(profile.configuration)
                                and grouping == profile.configuration["clustered_light_shading"]["grouping"]
                        ])

                        if row == 0:
                            samples = np.sum(profile.samples.basic_buffers[:, :, 1], 1)
                        if row == 1:
                            samples = np.sum(profile.samples.cluster_buffers[:, :, 0], 1)
                        if row == 2:
                            samples = np.sum(profile.samples.cluster_buffers[:, :, 1], 1)
                        ax.plot(samples, color=color_palette(color_base * 4 + color_offset), linestyle=linestyle, label="{} {}".format(grouping, projection.short_name()))

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/stereo_atemporal_{}.pdf'.format(scene_name), format='pdf')

profile_dir_regex = re.compile(r"^(suntem|bistro)_[ \d]{7}_(ortho|persp)_[ \d]{4}$");
profiles_0 = load_profiles("../profiling", profile_dir_regex);

generate_tune_plots(profiles_0)
generate_stackplots(profiles_0)
generate_ortho_vs_persp_plots(profiles_0)

stereo_profile_dir_regex = re.compile(r"^stereo_(suntem|bistro)_\d{7}_(indi|encl)_(ortho|persp)_\d{4}$")
stereo_profiles_0 = load_profiles("../profiling", stereo_profile_dir_regex)

generate_stereo_plots(stereo_profiles_0)

plt.show()
