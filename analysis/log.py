import matplotlib.pyplot as plt
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

profiling_dir = "../profiling/2019-09-17_16-04-01/"
with open(profiling_dir + "samples.bin", "rb") as f:
    run_count = read_u64(f)
    frame_count = read_u64(f)
    sample_count = read_u64(f)
    cluster_buffer_count = read_u64(f)
    field_count = 4

    stamp_count = run_count * frame_count * sample_count * field_count
    stamp_shape = (run_count, frame_count, sample_count, field_count)

    sample_names = []
    for sample_index in range(0, sample_count):
        sample_names.append(read_padded_string(f))

    stamps = np.reshape(np.fromfile(f, dtype='uint64', count = stamp_count), stamp_shape)

    cluster_buffers = np.reshape(np.fromfile(f, dtype='uint32', count = frame_count*cluster_buffer_count*68), (frame_count, cluster_buffer_count, 68));

deltas = np.subtract(stamps[:, :, :, [1, 3]], stamps[:, :, :, [0, 2]])

print(np.shape(deltas))
print(np.shape(cluster_buffers))

# samples = np.squeeze(np.median(deltas, axis = 0, keepdims = True));

def ceil_div(n, d):
    return (n + d - 1) // d


my_figsize = (20.00, 11.25);
my_dpi = 96;

row_count = 5

fig, subs = plt.subplots(ceil_div(sample_count, row_count), row_count, sharex = True, sharey = True, squeeze = False, figsize = my_figsize, dpi = my_dpi)
for sample_index in range(0, sample_count):
    sub = subs[sample_index // row_count, sample_index % row_count]
    sub.set_xlabel("frame");
    sub.set_ylim(0, 120000);
    sub.set_ylabel("ns");

    # Omit the first run. Take all the frames.
    current_samples = np.transpose(deltas[1:, :, sample_index, 0])
    sub.plot(current_samples, color = (0, 0, 0, 0.1))
    sub.plot(np.median(current_samples, axis = 1, keepdims = True))

    sub.title.set_text(sample_names[sample_index])
plt.savefig(profiling_dir + "cpu.png", dpi=my_dpi)

fig, subs = plt.subplots(ceil_div(sample_count, row_count), row_count, sharex = True, sharey = True, squeeze = False, figsize = my_figsize, dpi = my_dpi)
for sample_index in range(0, sample_count):
    sub = subs[sample_index // row_count, sample_index % row_count]
    sub.set_xlabel("frame");
    sub.set_ylim(0, 500000);
    sub.set_ylabel("ns");

    # Omit the first run. Take all the frames.
    current_samples = np.transpose(deltas[1:, :, sample_index, 1])
    sub.plot(current_samples, color = (0, 0, 0, 0.1))
    sub.plot(np.median(current_samples, axis = 1, keepdims = True))

    sub.title.set_text(sample_names[sample_index])
plt.savefig(profiling_dir + "gpu.png", dpi=my_dpi)

# plt.show()
