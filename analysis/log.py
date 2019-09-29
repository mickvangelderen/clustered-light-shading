import matplotlib.pyplot as plt
import numpy as np
from profiling_data import ProfilingData

profiling_dir = "../profiling/2019-09-29_12-10-51/"
pd = ProfilingData(profiling_dir)

deltas = np.subtract(pd.stamps[:, :, :, [1, 3]], pd.stamps[:, :, :, [0, 2]])

print(np.shape(deltas))
print(np.shape(pd.cluster_buffers))

def ceil_div(n, d):
    return (n + d - 1) // d

my_figsize = (20.00, 11.25);
my_dpi = 96;

row_count = 5

fig, subs = plt.subplots(ceil_div(pd.sample_count, row_count), row_count, sharex = True, sharey = True, squeeze = False, figsize = my_figsize, dpi = my_dpi)
for sample_index in range(0, pd.sample_count):
    sub = subs[sample_index // row_count, sample_index % row_count]
    sub.set_xlabel("frame");
    sub.set_ylim(0, 120000);
    sub.set_ylabel("ns");

    # Omit the first run. Take all the frames.
    current_samples = np.transpose(deltas[1:, :, sample_index, 0])
    sub.plot(current_samples, color = (0, 0, 0, 0.1))
    sub.plot(np.median(current_samples, axis = 1, keepdims = True))

    sub.title.set_text(pd.sample_names[sample_index])
plt.savefig(profiling_dir + "cpu.png", dpi=my_dpi)

fig, subs = plt.subplots(ceil_div(pd.sample_count, row_count), row_count, sharex = True, sharey = True, squeeze = False, figsize = my_figsize, dpi = my_dpi)
for sample_index in range(0, pd.sample_count):
    sub = subs[sample_index // row_count, sample_index % row_count]
    sub.set_xlabel("frame");
    sub.set_ylim(0, 500000);
    sub.set_ylabel("ns");

    # Omit the first run. Take all the frames.
    current_samples = np.transpose(deltas[1:, :, sample_index, 1])
    sub.plot(current_samples, color = (0, 0, 0, 0.1))
    sub.plot(np.median(current_samples, axis = 1, keepdims = True))

    sub.title.set_text(pd.sample_names[sample_index])
plt.savefig(profiling_dir + "gpu.png", dpi=my_dpi)

# plt.show()
