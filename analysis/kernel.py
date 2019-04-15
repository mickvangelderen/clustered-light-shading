from mpl_toolkits.mplot3d import Axes3D  # noqa: F401 unused import

import matplotlib.pyplot as plt
import numpy as np

dense = np.reshape(np.fromfile('../unit_sphere_dense.bin', dtype='float32'), (-1, 3));
volum = np.reshape(np.fromfile('../unit_sphere_volume.bin', dtype='float32'), (-1, 3));
surfa = np.reshape(np.fromfile('../unit_sphere_surface.bin', dtype='float32'), (-1, 3));

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
ax.set_xlim(-1.0, 1.0);
ax.set_ylim(-1.0, 1.0);
ax.set_zlim(-1.0, 1.0);
ax.scatter(dense[:, 0], dense[:, 1], dense[:, 2]);

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
ax.set_xlim(-1.0, 1.0);
ax.set_ylim(-1.0, 1.0);
ax.set_zlim(-1.0, 1.0);
ax.scatter(volum[:, 0], volum[:, 1], volum[:, 2]);

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
ax.set_xlim(-1.0, 1.0);
ax.set_ylim(-1.0, 1.0);
ax.set_zlim(-1.0, 1.0);
ax.scatter(surfa[:, 0], surfa[:, 1], surfa[:, 2]);

plt.show()
