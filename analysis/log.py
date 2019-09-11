import matplotlib.pyplot as plt
import numpy as np

blob = np.fromfile('../samples.bin', dtype='uint64')

run_count = blob[0]
frame_count = blob[1]
sample_count = blob[2]
field_count = 4

stamps = np.reshape(blob[3:], (run_count, frame_count, sample_count, field_count))

deltas = np.subtract(stamps[:, :, :, [1, 3]], stamps[:, :, :, [0, 2]])

print(np.shape(deltas))

# samples = np.squeeze(np.median(deltas, axis = 0, keepdims = True));

for sample_index in range(0, sample_count):
    fig, subs = plt.subplots(1, 1, squeeze = False)
    sub = subs[0, 0]
    for run_index in range(0, run_count):
        sub.plot(deltas[run_index, :, sample_index, 0])
    # axe.semilogy();
    # legend = ['sim', 'pos', 'ren'];
    # fig.legend(legend);
    plt.show()


