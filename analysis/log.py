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

with open("../samples.bin", "rb") as f:
    run_count = read_u64(f)
    frame_count = read_u64(f)
    sample_count = read_u64(f)
    field_count = 4

    stamp_count = run_count * frame_count * sample_count * field_count
    stamp_shape = (run_count, frame_count, sample_count, field_count)

    sample_names = []
    for sample_index in range(0, sample_count):
        sample_names.append(read_padded_string(f))

    stamps = np.reshape(np.fromfile(f, dtype='uint64', count = stamp_count), stamp_shape)

deltas = np.subtract(stamps[:, :, :, [1, 3]], stamps[:, :, :, [0, 2]])

print(np.shape(deltas))

# samples = np.squeeze(np.median(deltas, axis = 0, keepdims = True));

def ceil_div(n, d):
    return (n + d - 1) // d

row_count = 5

fig, subs = plt.subplots(ceil_div(sample_count, row_count), row_count, squeeze = False, figsize = (20, 20))
for sample_index in range(0, sample_count):
    sub = subs[sample_index // row_count, sample_index % row_count]
    for run_index in range(0, run_count):
        sub.plot(deltas[run_index, 1:, sample_index, 1])
    sub.title.set_text(sample_names[sample_index])

    # axe.semilogy();
    # fig.legend(legend);

    # legend = ['sim', 'pos', 'ren'];
plt.show()

