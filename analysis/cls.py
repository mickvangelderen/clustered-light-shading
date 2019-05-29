from mpl_toolkits.mplot3d import Axes3D  # noqa: F401 unused import

import matplotlib.pyplot as plt
import numpy as np

u32s = np.fromfile('../cls.bin', dtype=np.uint32);

# [u32; 4]
dims = u32s[0:4];
print(dims)

# [u32; dims[3]] after vec4 + mat4 + mat4
lens = u32s[36::dims[3]].reshape((dims[2], dims[1], dims[0]));
print(lens.shape)

fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
ax.set_xlim(-1.0, dims[0] + 1.0)
ax.set_ylim(-1.0, dims[1] + 1.0)
ax.set_zlim(-1.0, dims[2] + 1.0)
Z, Y, X = np.meshgrid(np.arange(dims[2]), np.arange(dims[1]), np.arange(dims[0]), indexing="ij")

filt = np.where(lens > 0)
print(filt)
ax.scatter(
    Z[filt],
    Y[filt],
    X[filt],
)
plt.show()

# fig = plt.figure()
# ax = fig.add_subplot(111, projection='3d')
# ax.set_xlim(-1.0, 1.0);
# ax.set_ylim(-1.0, 1.0);
# ax.set_zlim(-1.0, 1.0);
# ax.scatter(volum[:, 0], volum[:, 1], volum[:, 2]);

# fig = plt.figure()
# ax = fig.add_subplot(111, projection='3d')
# ax.set_xlim(-1.0, 1.0);
# ax.set_ylim(-1.0, 1.0);
# ax.set_zlim(-1.0, 1.0);
# ax.scatter(refle[:, 0], refle[:, 1], refle[:, 2]);

# fig = plt.figure()
# ax = fig.add_subplot(111, projection='3d')
# ax.set_xlim(-1.0, 1.0);
# ax.set_ylim(-1.0, 1.0);
# ax.set_zlim(-1.0, 1.0);
# ax.scatter(surfa[:, 0], surfa[:, 1], surfa[:, 2]);

# fig = plt.figure()
# ax = fig.add_subplot(111, projection='3d')
# ax.set_xlim(-1.0, 1.0);
# ax.set_ylim(-1.0, 1.0);
# ax.set_zlim(-1.0, 1.0);
# ax.scatter(surfr[:, 0], surfr[:, 1], surfr[:, 2]);

# plt.show()
