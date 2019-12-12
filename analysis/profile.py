import os
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

    def __eq__(self, other):
        return isinstance(other, type(self)) and (self.count, self.attenuation) == (other.count, other.attenuation)

    def __hash__(self):
        return hash((self.count, self.attenuation))

class Profile:
    def __init__(self, directory):
        self.directory = directory
        self.configuration = toml.load(os.path.join(directory, "configuration.toml"))
        self.samples = ProfileSamples(directory)

def load_profiles(profiling_directory, regex):
    with os.scandir(profiling_directory) as it:
        directories = [os.path.join(profiling_directory, entry.name) for entry in it if entry.is_dir() and regex.match(entry.name)]

    return [Profile(directory) for directory in directories]

profile_dir_regex = re.compile(r"^(suntem|bistro)_[ \d]{7}_(ortho|persp)_[ \d]{4}$");
profiles_0 = load_profiles("../profiling", profile_dir_regex);

sample_names = ["/frame", "/frame/cluster", "/frame/basic"]

print("thesis.textwidth = {}in, thesis.dpi = {}".format(thesis.textwidth, thesis.dpi))

def gridspec_box(l, r, b, t, w, h):
    return {
        "left": l/w,
        "right": (w - r)/w,
        "bottom": b/h,
        "top": (h - t)/h,
    }

for (scene_name, scene_path) in [
    ("bistro", "bistro/Bistro_Exterior.bin"),
    ("suntem", "sun_temple/SunTemple.bin"),
]:

    profiles_1 = [profile for profile in profiles_0 if
                        profile.configuration["global"]["scene_path"] == scene_path]

    lightings = sorted({ Lighting(profile.configuration) for profile in profiles_1 }, key = lambda x: x.count)

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
                frame_sample_index = profile.samples.sample_names.index(sample_name)
                # GPU Samples from all frames except the first
                run_samples = profile.samples.deltas[1:, :, frame_sample_index, 1]
                samples = np.nanmin(run_samples, axis = 0) / 1000000.0
                size = profile.configuration["clustered_light_shading"]["orthographic_sides"]
                size_x = size["x"]
                assert size_x == size["y"]
                assert size_x == size["z"]
                ax.plot(samples, label="ortho {}".format(size_x))

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
                frame_sample_index = profile.samples.sample_names.index(sample_name)
                # GPU Samples from all frames except the first
                run_samples = profile.samples.deltas[1:, :, frame_sample_index, 1]
                samples = np.nanmin(run_samples, axis = 0) / 1000000.0
                size = profile.configuration["clustered_light_shading"]["perspective_pixels"]
                size_x = size["x"]
                assert size_x == size["y"]
                ax.plot(samples, label="persp {}".format(size_x))

            ax.legend(loc = 'upper left')

    fig.align_ylabels(axes)

    fig.savefig('../../thesis/media/tune_persp_{}.pdf'.format(scene_name), format='pdf')

plt.show()

# 1. for ortho, persp: select a size based on frame time
# 2. compare best persp and best ortho in (clustering vs shading time), cluster count
