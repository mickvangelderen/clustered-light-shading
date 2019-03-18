import matplotlib.pyplot as plt
import numpy as np

raw_data = np.reshape(np.fromfile('../log/log.bin', dtype='uint64'), (-1, 4));
print(np.shape(raw_data));
diff_data = np.diff(raw_data);
print(np.shape(diff_data));

# fig, axes = plt.subplots(3, 1, sharex=True, sharey=True);
# for i in range(0, 3):
    # axe = axes[i];
    # axe.plot(diff_data[:, i]);
    # axe.semilogy();

legend = ['sim', 'pos', 'ren'];
axe = plt.plot(diff_data);
plt.legend(legend);

plt.show()
