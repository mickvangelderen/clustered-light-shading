from mpl_toolkits.mplot3d import Axes3D  # noqa: F401 unused import

import matplotlib.pyplot as plt
import numpy as np

raw_data = np.reshape(np.fromfile('../target/debug/build/renderer-b253821093a8a9ff/out/hbao_kernel.bin', dtype='float32'), (-1, 4));
print(np.shape(raw_data));

# Samples

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')

ax.set_xlim(-1.0, 1.0);
ax.set_ylim(-1.0, 1.0);
ax.set_zlim(-1.0, 1.0);

ax.scatter(raw_data[0:512, 0], raw_data[0:512, 1], raw_data[0:512, 2]);

# Normals
fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')

ax.set_xlim(-1.0, 1.0);
ax.set_ylim(-1.0, 1.0);
ax.set_zlim(-1.0, 1.0);

ax.scatter(raw_data[512:1024, 0], raw_data[512:1024, 1], raw_data[512:1024, 2]);

plt.show()
