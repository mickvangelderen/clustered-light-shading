import bpy
import os

file_path = os.path.splitext(bpy.data.filepath)[0] + "-lights.rs";

if not file_path:
    raise Exception("Blend file is not saved")

file = open(file_path, 'w')
file.write("[\n")

def fmt_rgb(v):
    return "RGB::new({:.4f}, {:.4f}, {:.4f})".format(v[0], v[1], v[2])

def fmt_point3(v):
    return "Point3::new({:.4f}, {:.4f}, {:.4f})".format(v[0], v[2], -v[1])

for light in (o for o in (o for o in bpy.data.objects if o.type == 'LAMP') if o.data.type == 'POINT'):
    file.write("PointLight {\n")
    file.write("  ambient: {},\n".format(fmt_rgb([0.2, 0.2, 0.2])))
    file.write("  diffuse: {},\n".format(fmt_rgb(light.data.color * light.data.energy)))
    file.write("  specular: {},\n".format(fmt_rgb(light.data.color)))
    file.write("  pos_in_pnt: {},\n".format(fmt_point3(light.location)))
    file.write("  attenuation: AttenCoefs {{ constant: {:.4f}, linear: {:.4f}, quadratic: {:.4f} }},\n".format(light.data.constant_coefficient, light.data.linear_coefficient, light.data.quadratic_coefficient))
    file.write("},\n")

file.write("]\n")
file.close()
