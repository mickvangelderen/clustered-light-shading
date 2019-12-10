import os
import re
import toml
import numpy as np
from profile_samples import ProfileSamples
import matplotlib.pyplot as plt

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
profiles = load_profiles("../profiling", profile_dir_regex);

current_profiles = [profile for profile in profiles if
                    profile.configuration["global"]["scene_path"] == "bistro/Bistro_Exterior.bin"]

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

lightings = sorted({ Lighting(profile.configuration) for profile in current_profiles }, key = lambda v: v.count);

fig, axes = plt.subplots(1, len(lightings), sharey = 'row', squeeze=False)

for col, lighting in enumerate(lightings):
    current_lighting_profiles = [p for p in current_profiles if lighting == Lighting(p.configuration)]

    ax = axes[0, col]
    ax.set_title("Bistro {} lights i = {}, r1 = {:.2f}".format(
        lighting.count,
        lighting.attenuation.i,
        lighting.attenuation.r1
    ))

    ortho_profiles = [p for p in current_lighting_profiles if p.configuration["clustered_light_shading"]["projection"] == "Orthographic"]

    ortho_profiles.sort(key = lambda p: p.configuration["clustered_light_shading"]["orthographic_sides"]["x"])

    for profile in ortho_profiles:
        frame_sample_index = profile.samples.sample_names.index("/frame")
        # GPU Samples from all frames except the first
        run_samples = profile.samples.deltas[1:, :, frame_sample_index, 1]
        samples = np.nanmin(run_samples, axis = 0) / 1000000.0
        size = profile.configuration["clustered_light_shading"]["orthographic_sides"]
        size_x = size["x"]
        assert size_x == size["y"]
        assert size_x == size["z"]
        ax.plot(samples, label="ortho {}".format(size_x))

    persp_profiles = [p for p in current_lighting_profiles if p.configuration["clustered_light_shading"]["projection"] == "Perspective"]

    persp_profiles.sort(key = lambda p: p.configuration["clustered_light_shading"]["perspective_pixels"]["x"])
    for profile in persp_profiles:
        frame_sample_index = profile.samples.sample_names.index("/frame")
        # GPU Samples from all frames except the first
        run_samples = profile.samples.deltas[1:, :, frame_sample_index, 1]
        samples = np.nanmin(run_samples, axis = 0) / 1000000.0
        size = profile.configuration["clustered_light_shading"]["perspective_pixels"]
        size_x = size["x"]
        assert size_x == size["y"]
        ax.plot(samples, label="persp {}".format(size_x))

    ax.legend()

plt.show()
