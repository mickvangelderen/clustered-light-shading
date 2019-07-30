import numpy as np

wld_to_cam = np.array([
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, -1.0],
    [0.0, 0.0, 1.0, -1.5],
    [0.0, 0.0, 0.0, 1.0],
])
cam_to_wld = np.linalg.inv(wld_to_cam)

cam_to_clp = np.array([
    [0.5624, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0025, 0.1003],
    [0.0, 0.0, -1.0, 0.0],
])
clp_to_cam = np.linalg.inv(cam_to_clp)

wld_to_cls = np.array([
    [0.6680, 0.0, 0.0, 47.5],
    [0.0, 0.6750, 0.0, 26.33],
    [0.0, 0.0, 0.6767, 26.05],
    [0.0, 0.0, 0.0, 1.0],
])
cls_to_wld = np.linalg.inv(wld_to_cls)

clp_to_cls = np.linalg.multi_dot([wld_to_cls, cam_to_wld, clp_to_cam])

print(np.dot(cam_to_clp, np.array([0.0, 0.0, -20.0, 1.0])));
with np.printoptions(suppress=True, formatter={'float': '{:>8.1f}'.format}):
    for z in [0.0, 0.0025, 1.0]:
        for y in [-1.0, 0.0, 1.0]:
            for x in [-1.0, 0.0, 1.0]:
                pos_in_ndc = np.array([x, y, z, 1.0])
                # p = np.dot(clp_to_cam, pos_in_ndc)
                # pos_in_cam = p/p[3]
                # print(pos_in_ndc[0:3], pos_in_cam[0:3])
                p = np.dot(clp_to_cls, pos_in_ndc)
                pos_in_cls = p/p[3]
                print(pos_in_ndc[0:3], pos_in_cls[0:3])
