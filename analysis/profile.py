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
            return "ortho {}m$^3$".format(self.x)
        else:
            return "ortho {}x{}x{}m$^3$".format(self.x, self.y, self.z)

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
            return "persp {}px$^2$".format(self.x)
        else:
            return "persp {}x{}px$^2$".format(self.x, self.y)

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

def gridspec_box(l, r, b, t, w, h):
    return {
        "left": l/w,
        "right": (w - r)/w,
        "bottom": b/h,
        "top": (h - t)/h,
    }

profile_dir_regex = re.compile(r"^(suntem|bistro)_[ \d]{7}_(ortho|persp)_[ \d]{4}$");
profiles_0 = load_profiles("../profiling", profile_dir_regex);

lightings = sorted({ Lighting(profile.configuration) for profile in profiles_0 }, key = lambda x: x.count)

scenes = [
    ("bistro", "bistro/Bistro_Exterior.bin"),
    ("suntem", "sun_temple/SunTemple.bin"),
]

tuned_projections = [
    ClusteringProjectionPerspective(64, 64),
    ClusteringProjectionOrthographic(4.0, 4.0, 4.0),
]

def generate_tune_plots(profiles_0):
    sample_names = ["/frame", "/frame/cluster", "/frame/basic"]

    for (scene_name, scene_path) in scenes:

        profiles_1 = [profile for profile in profiles_0 if
                            profile.configuration["global"]["scene_path"] == scene_path]

        # ortho

        fig, axes = plt.subplots(len(sample_names), len(lightings), sharex = 'col', squeeze=False, figsize = (thesis.textwidth, thesis.textwidth), dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.1, 0.5, 0.3, thesis.textwidth, thesis.textwidth)
        )

        for row, sample_name in enumerate(sample_names):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == len(sample_names):
                    ax.set_xlabel("frame")

                if col == 0:
                    ax.set_ylabel("{} (ms)".format(sample_name))

                profiles_2 = sorted([
                    profile for profile in profiles_1
                    if lighting == Lighting(profile.configuration)
                    and profile.configuration["clustered_light_shading"]["projection"] == "Orthographic"
                ], key = lambda profile: profile.configuration["clustered_light_shading"]["orthographic_sides"]["x"])

                for profile in profiles_2:
                    samples = profile.samples.min_gpu_samples_by_name(sample_name)
                    projection = ClusteringProjection.from_configuration(profile.configuration)
                    ax.plot(samples, label=projection.short_name())

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/tune_ortho_{}.pdf'.format(scene_name), format='pdf')

        # persp

        fig, axes = plt.subplots(len(sample_names), len(lightings), sharex = 'col', squeeze=False, figsize = (thesis.textwidth, thesis.textwidth), dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.1, 0.5, 0.3, thesis.textwidth, thesis.textwidth)
        )

        for row, sample_name in enumerate(sample_names):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == len(sample_names):
                    ax.set_xlabel("frame")

                if col == 0:
                    ax.set_ylabel("{} (ms)".format(sample_name))

                profiles_2 = sorted([
                    profile for profile in profiles_1
                    if lighting == Lighting(profile.configuration)
                    and profile.configuration["clustered_light_shading"]["projection"] == "Perspective"
                ], key = lambda p: p.configuration["clustered_light_shading"]["perspective_pixels"]["x"])

                for profile in profiles_2:
                    samples = profile.samples.min_gpu_samples_by_name(sample_name)
                    projection = ClusteringProjection.from_configuration(profile.configuration)
                    ax.plot(samples, label=projection.short_name())

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/tune_persp_{}.pdf'.format(scene_name), format='pdf')

        for lighting in lightings:
            for (scene_name, frames) in [
                ("bistro", [300, 580]),
                ("suntem", [250, 680]),
            ]:
                for frame in frames:
                    i = "../profiling/{}_{:07}_ortho_0400/frames/{}.bmp".format(scene_name, lighting.count, frame)
                    o = "../../thesis/media/{}_{:07}_ortho_0400_{}.jpg".format(scene_name, lighting.count, frame)
                    subprocess.run(["convert", "-format", "jpg", "-quality", "95", i, o])

def generate_stackplots(profiles_0):
    for (scene_name, scene_path) in scenes:
        fig, axes = plt.subplots(len(tuned_projections), len(lightings), sharex = 'col', sharey = 'all', squeeze=False, figsize = (thesis.textwidth, thesis.textwidth), dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.1, 0.5, 0.3, thesis.textwidth, thesis.textwidth)
        )

        profiles_1 = [profile for profile in profiles_0 if
                      profile.configuration["global"]["scene_path"] == scene_path]

        for row, projection in enumerate(tuned_projections):

            profiles_2 = [profile for profile in profiles_1
                          if projection == ClusteringProjection.from_configuration(profile.configuration)]

            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                profiles_3 = [profile for profile in profiles_2
                              if lighting == Lighting(profile.configuration)]

                assert 1 == len(profiles_3)

                profile = profiles_3[0]

                frame_samples = profile.samples.min_gpu_samples_by_name("/frame")
                cluster_samples = profile.samples.min_gpu_samples_by_name("/frame/cluster")
                basic_samples = profile.samples.min_gpu_samples_by_name("/frame/basic")
                misc_samples = frame_samples - cluster_samples - basic_samples
                stacked_samples = np.vstack([basic_samples, cluster_samples, misc_samples])
                stacked_labels = ["shading", "clustering", "miscellaneous"]

                x0 = 0
                x1 = stacked_samples.shape[1]

                ax.stackplot(np.arange(x0, x1), stacked_samples, labels=stacked_labels)
                ax.set_xlim(x0, x1)

                if col == 0:
                    ax.legend(title = projection.short_name())

                if row == 0:
                    ax.set_title(lighting.short_name())

            fig.align_ylabels(axes)

            fig.savefig('../../thesis/media/stack_{}.pdf'.format(scene_name), format='pdf')

def generate_ortho_vs_persp_plots(profiles_0):
    sample_names = ["/frame", "/frame/cluster", "/frame/basic"]

    for (scene_name, scene_path) in [
        ("bistro", "bistro/Bistro_Exterior.bin"),
        ("suntem", "sun_temple/SunTemple.bin"),
    ]:

        profiles_1 = [profile for profile in profiles_0 if
                            profile.configuration["global"]["scene_path"] == scene_path]

        fig, axes = plt.subplots(len(sample_names), len(lightings), sharex = 'col', squeeze=False, figsize = (thesis.textwidth, thesis.textwidth), dpi = thesis.dpi,
            gridspec_kw = gridspec_box(0.6, 0.1, 0.5, 0.3, thesis.textwidth, thesis.textwidth)
        )

        for row, sample_name in enumerate(sample_names):
            for col, lighting in enumerate(lightings):
                ax = axes[row, col]

                if row == 0:
                    ax.set_title("{} lights (r1 = {:.2f})".format(
                        lighting.count,
                        lighting.attenuation.r1
                    ))

                if row + 1 == len(sample_names):
                    ax.set_xlabel("frame")

                if col == 0:
                    ax.set_ylabel("{} (ms)".format(sample_name))

                for projection in tuned_projections:
                    profiles_2 = [profile for profile in profiles_1
                                  if lighting == Lighting(profile.configuration)
                                  and projection == ClusteringProjection.from_configuration(profile.configuration)]

                    assert 1 == len(profiles_2)

                    profile = profiles_2[0]

                    samples = profile.samples.min_gpu_samples_by_name(sample_name)
                    ax.plot(samples, label=projection.short_name())

                if row == 0 and col == 0:
                    ax.legend(loc = 'upper left')

        fig.align_ylabels(axes)

        fig.savefig('../../thesis/media/ortho_vs_persp_{}.pdf'.format(scene_name), format='pdf')

generate_tune_plots(profiles_0)
generate_stackplots(profiles_0)
generate_ortho_vs_persp_plots(profiles_0)

# plt.show()
