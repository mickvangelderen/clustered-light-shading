import os
import struct
import numpy as np

struct_u64 = struct.Struct('Q');

def read_u64(f):
    return struct_u64.unpack(f.read(struct_u64.size))[0]

def read_padded_string(f):
    byte_count = read_u64(f)
    string = f.read(byte_count).decode("utf-8")
    if (byte_count % 8) != 0:
        # Read padding, could use seek maybe?
        pad_count = 8 - (byte_count % 8)
        f.read(pad_count)
    return string

class ProfileSamples:
    def __init__(self, profiling_dir):
        with open(os.path.join(profiling_dir, "samples.bin"), "rb") as f:
            self.run_count = read_u64(f)
            self.frame_count = read_u64(f)
            self.sample_count = read_u64(f)
            self.cluster_buffer_count = read_u64(f)
            self.basic_buffer_count = read_u64(f)
            self.field_count = 4

            stamp_count = self.run_count * self.frame_count * self.sample_count * self.field_count
            stamp_shape = (self.run_count, self.frame_count, self.sample_count, self.field_count)

            self.sample_names = []
            for sample_index in range(0, self.sample_count):
                self.sample_names.append(read_padded_string(f))

            self.stamps = np.reshape(np.fromfile(f, dtype='uint64', count = stamp_count), stamp_shape)

            self.deltas = np.subtract(self.stamps[:, :, :, [1, 3]], self.stamps[:, :, :, [0, 2]])

            cluster_buffer_u32_size = 256*4

            self.cluster_buffers = np.reshape(
                np.fromfile(f, dtype='uint32', count = self.frame_count*self.cluster_buffer_count*cluster_buffer_u32_size),
                (self.frame_count, self.cluster_buffer_count, cluster_buffer_u32_size)
            );

            basic_buffer_u32_size = 2

            self.basic_buffers = np.reshape(
                np.fromfile(f, dtype='uint32', count = self.frame_count*self.basic_buffer_count*basic_buffer_u32_size),
                (self.frame_count, self.basic_buffer_count, basic_buffer_u32_size)
            );

    def min_gpu_samples_by_name(self, sample_name):
        frame_sample_index = self.sample_names.index(sample_name)
        samples = self.deltas[:, :, frame_sample_index, 1]
        return np.nanmin(samples, axis = 0) / 1000000.0

    def sum_visible_clusters(self):
        return np.sum(self.cluster_buffers[:, :, 0], axis=1)

    def sum_light_indices(self):
        return np.sum(self.cluster_buffers[:, :, 1], axis=1)

    def sum_lighting_operations(self):
        return np.sum(self.basic_buffers[:, :, 0], axis=1)

    def sum_shading_operations(self):
        return np.sum(self.basic_buffers[:, :, 1], axis=1)
