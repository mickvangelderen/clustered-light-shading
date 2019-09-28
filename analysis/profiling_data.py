import numpy as np
import struct

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

profiling_dir = "../profiling/2019-09-28_12-28-49/"

class ProfilingData:
    def __init__(self, profiling_dir):
        with open(profiling_dir + "samples.bin", "rb") as f:
            self.run_count = read_u64(f)
            self.frame_count = read_u64(f)
            self.sample_count = read_u64(f)
            self.cluster_buffer_count = read_u64(f)
            self.field_count = 4

            stamp_count = self.run_count * self.frame_count * self.sample_count * self.field_count
            stamp_shape = (self.run_count, self.frame_count, self.sample_count, self.field_count)

            self.sample_names = []
            for sample_index in range(0, self.sample_count):
                self.sample_names.append(read_padded_string(f))

            self.stamps = np.reshape(np.fromfile(f, dtype='uint64', count = stamp_count), stamp_shape)

            self.cluster_buffers = np.reshape(np.fromfile(f, dtype='uint32', count = self.frame_count*self.cluster_buffer_count*68), (self.frame_count, self.cluster_buffer_count, 68));

