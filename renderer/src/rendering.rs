use crate::*;

// Capabilities.

macro_rules! capability_declaration {
    () => {
        r"
#version 430 core
#extension GL_ARB_gpu_shader5 : enable
#extension GL_NV_gpu_shader5 : enable
"
    };
}

// Constants.

pub const POINT_LIGHT_CAPACITY: u32 = 8;

macro_rules! constant_declaration {
    () => {
        r"
#define POINT_LIGHT_CAPACITY 8
"
    };
}

// Storage buffer bindings.

pub const GLOBAL_DATA_BINDING: u32 = 0;
pub const VIEW_DATA_BINDING: u32 = 1;
pub const MATERIAL_DATA_BINDING: u32 = 2;
pub const AO_SAMPLE_BUFFER_BINDING: u32 = 3;
pub const LIGHTING_BUFFER_BINDING: u32 = 4;
pub const CLS_BUFFER_BINDING: u32 = 5;

macro_rules! buffer_binding_declaration {
    () => {
        r"
#define GLOBAL_DATA_BINDING 0
#define VIEW_DATA_BINDING 1
#define MATERIAL_DATA_BINDING 2
#define AO_SAMPLE_BUFFER_BINDING 3
#define LIGHTING_BUFFER_BINDING 4
#define CLS_BUFFER_BINDING 5
"
    };
}

// Attribute locations.

pub const VS_POS_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(0) };
pub const VS_POS_IN_TEX_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(1) };
pub const VS_NOR_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(2) };
pub const VS_TAN_IN_OBJ_LOC: gl::AttributeLocation = unsafe { gl::AttributeLocation::new_unchecked(3) };

macro_rules! attribute_location_declaration {
    () => {
        r"
#define VS_POS_IN_OBJ_LOC 0
#define VS_POS_IN_TEX_LOC 1
#define VS_NOR_IN_OBJ_LOC 2
#define VS_TAN_IN_OBJ_LOC 3
"
    };
}

pub const COMMON_DECLARATION: &'static str = concat!(
    capability_declaration!(),
    constant_declaration!(),
    buffer_binding_declaration!(),
    attribute_location_declaration!(),
);

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct GlobalData {
    pub light_pos_from_wld_to_cam: Matrix4<f32>,
    pub light_pos_from_cam_to_wld: Matrix4<f32>,

    pub light_pos_from_cam_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_cam: Matrix4<f32>,

    pub light_pos_from_wld_to_clp: Matrix4<f32>,
    pub light_pos_from_clp_to_wld: Matrix4<f32>,

    pub time: f64,
}

pub const GLOBAL_DATA_DECLARATION: &'static str = r"
layout(std140, binding = GLOBAL_DATA_BINDING) uniform GlobalData {
    mat4 light_pos_from_wld_to_cam;
    mat4 light_pos_from_cam_to_wld;

    mat4 light_pos_from_cam_to_clp;
    mat4 light_pos_from_clp_to_cam;

    mat4 light_pos_from_wld_to_clp;
    mat4 light_pos_from_clp_to_wld;

    double time;
};
";

#[derive(Debug, Copy, Clone)]
pub struct GlobalResources {
    buffer_name: gl::BufferName,
}

impl GlobalResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            GlobalResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.bind_buffer_base(gl::UNIFORM_BUFFER, GLOBAL_DATA_BINDING, self.buffer_name);
        }
    }

    #[inline]
    pub fn write(&self, gl: &gl::Gl, data: &GlobalData) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.value_as_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct ViewData {
    pub pos_from_wld_to_cam: Matrix4<f32>,
    pub pos_from_cam_to_wld: Matrix4<f32>,

    pub pos_from_cam_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_cam: Matrix4<f32>,

    pub pos_from_wld_to_clp: Matrix4<f32>,
    pub pos_from_clp_to_wld: Matrix4<f32>,

    pub cam_pos_in_lgt: Vector4<f32>,
    pub light_dir_in_cam: Vector4<f32>,
}

pub const VIEW_DATA_DECLARATION: &'static str = r"
layout(std140, binding = VIEW_DATA_BINDING) uniform ViewData {
    mat4 pos_from_wld_to_cam;
    mat4 pos_from_cam_to_wld;

    mat4 pos_from_cam_to_clp;
    mat4 pos_from_clp_to_cam;

    mat4 pos_from_wld_to_clp;
    mat4 pos_from_clp_to_wld;

    vec4 cam_pos_in_lgt;
    vec4 light_dir_in_cam;
};
";

#[derive(Debug)]
pub struct ViewData0 {
    pub pos_from_wld_to_cam: Matrix4<f64>,
    pub pos_from_cam_to_wld: Matrix4<f64>,

    pub pos_from_cam_to_clp: Matrix4<f64>,
    pub pos_from_clp_to_cam: Matrix4<f64>,

    pub cam_pos_in_lgt: Vector3<f64>,
}

impl ViewData0 {
    pub fn into_view_data(self, global_data: &GlobalData) -> ViewData {
        let ViewData0 {
            pos_from_wld_to_cam,
            pos_from_cam_to_wld,

            pos_from_cam_to_clp,
            pos_from_clp_to_cam,

            cam_pos_in_lgt,
        } = self;

        let pos_from_wld_to_clp = pos_from_cam_to_clp * pos_from_wld_to_cam;
        let pos_from_clp_to_wld = pos_from_cam_to_wld * pos_from_clp_to_cam;

        let light_pos_from_cam_to_wld = global_data.light_pos_from_cam_to_wld.cast::<f64>().unwrap();
        let light_dir_in_cam = self
            .pos_from_wld_to_cam
            .transform_vector(light_pos_from_cam_to_wld.transform_vector(Vector3::unit_z()));

        rendering::ViewData {
            pos_from_wld_to_cam: pos_from_wld_to_cam.cast().unwrap(),
            pos_from_cam_to_wld: pos_from_cam_to_wld.cast().unwrap(),

            pos_from_cam_to_clp: pos_from_cam_to_clp.cast().unwrap(),
            pos_from_clp_to_cam: pos_from_clp_to_cam.cast().unwrap(),

            pos_from_wld_to_clp: pos_from_wld_to_clp.cast().unwrap(),
            pos_from_clp_to_wld: pos_from_clp_to_wld.cast().unwrap(),

            cam_pos_in_lgt: cam_pos_in_lgt.cast().unwrap().extend(0.0),
            light_dir_in_cam: light_dir_in_cam.cast().unwrap().extend(0.0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ViewResources {
    buffer_name: gl::BufferName,
}

impl ViewResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            ViewResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind_index(&self, gl: &gl::Gl, index: usize) {
        unsafe {
            gl.bind_buffer_range(
                gl::UNIFORM_BUFFER,
                VIEW_DATA_BINDING,
                self.buffer_name,
                std::mem::size_of::<ViewData>() * index,
                std::mem::size_of::<ViewData>(),
            );
        }
    }

    /// Use when the data isn't laid out in memory consecutively.
    pub fn write_all_ref(&self, gl: &gl::Gl, data: &[&ViewData]) {
        unsafe {
            let total_bytes = data.len() * std::mem::size_of::<ViewData>();
            gl.named_buffer_reserve(self.buffer_name, total_bytes, gl::DYNAMIC_DRAW);
            for (index, &item) in data.iter().enumerate() {
                gl.named_buffer_sub_data(
                    self.buffer_name,
                    index * std::mem::size_of::<ViewData>(),
                    item.value_as_bytes(),
                );
            }
        }
    }

    #[inline]
    pub fn write_all(&self, gl: &gl::Gl, data: &[ViewData]) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.slice_to_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(256))]
pub struct MaterialData {
    pub shininess: f32,
}

pub const MATERIAL_DATA_DECLARATION: &'static str = r"
layout(std140, binding = MATERIAL_DATA_BINDING) uniform MaterialData {
    float shininess;
};
";

#[derive(Debug, Copy, Clone)]
pub struct MaterialResources {
    buffer_name: gl::BufferName,
}

impl MaterialResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            MaterialResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind_index(&self, gl: &gl::Gl, index: usize) {
        unsafe {
            gl.bind_buffer_range(
                gl::UNIFORM_BUFFER,
                MATERIAL_DATA_BINDING,
                self.buffer_name,
                std::mem::size_of::<MaterialData>() * index,
                std::mem::size_of::<MaterialData>(),
            );
        }
    }

    #[inline]
    pub fn write_all(&self, gl: &gl::Gl, data: &[MaterialData]) {
        unsafe {
            gl.named_buffer_data(self.buffer_name, data.slice_to_bytes(), gl::DYNAMIC_DRAW);
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CLSBufferHeader {
    pub dimensions: Vector4<u32>,
    pub pos_from_wld_to_cls: Matrix4<f32>,
    pub pos_from_cls_to_wld: Matrix4<f32>,
}

#[derive(Debug)]
pub struct CLSBuffer {
    pub header: CLSBufferHeader,
    pub body: Vec<[u32; crate::cls::MAX_LIGHTS_PER_CLUSTER]>,
}

pub const CLS_BUFFER_DECLARATION: &'static str = r"
layout(std430, binding = CLS_BUFFER_BINDING) buffer CLSBuffer {
    uvec4 cluster_dims;
    mat4 pos_from_wld_to_cls;
    mat4 pos_from_cls_to_wld;
    uint clusters[];
};
";

#[derive(Debug, Copy, Clone)]
pub struct CLSResources {
    buffer_name: gl::BufferName,
}

impl CLSResources {
    #[inline]
    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            CLSResources {
                buffer_name: gl.create_buffer(),
            }
        }
    }

    #[inline]
    pub fn bind(&self, gl: &gl::Gl) {
        unsafe {
            gl.bind_buffer_base(gl::SHADER_STORAGE_BUFFER, CLS_BUFFER_BINDING, self.buffer_name);
        }
    }

    #[inline]
    pub fn write(&self, gl: &gl::Gl, cls_buffer: &CLSBuffer) {
        unsafe {
            let header_bytes = cls_buffer.header.value_as_bytes();
            let body_bytes = cls_buffer.body.vec_as_bytes();
            let total_size = header_bytes.len() + body_bytes.len();
            gl.named_buffer_reserve(self.buffer_name, total_size, gl::STREAM_DRAW);
            gl.named_buffer_sub_data(self.buffer_name, 0, header_bytes);
            gl.named_buffer_sub_data(self.buffer_name, header_bytes.len(), body_bytes);
        }
    }
}

#[derive(Debug)]
pub struct ShaderSource {
    pub path: PathBuf,
    pub modified: ic::Modified,
}

impl ShaderSource {
    pub fn read(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum LightSpace {
    Wld = 1,
    Hmd = 2,
    Cam = 3,
}

impl LightSpace {
    pub fn source(self) -> &'static str {
        match self {
            LightSpace::Wld => "#define LIGHT_SPACE_WLD\n",
            LightSpace::Hmd => "#define LIGHT_SPACE_HMD\n",
            LightSpace::Cam => "#define LIGHT_SPACE_CAM\n",
        }
    }

    pub fn regex() -> Regex {
        Regex::new(r"\bLIGHT_SPACE_(WLD|HMD|CAM)\b").unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum RenderTechnique {
    Naive = 1,
    Tiled = 2,
    Clustered = 3,
}

impl RenderTechnique {
    pub fn source(self) -> &'static str {
        match self {
            RenderTechnique::Naive => "#define RENDER_TECHNIQUE_NAIVE\n",
            RenderTechnique::Tiled => "#define RENDER_TECHNIQUE_TILED\n",
            RenderTechnique::Clustered => "#define RENDER_TECHNIQUE_CLUSTERED\n",
        }
    }

    pub fn regex() -> Regex {
        Regex::new(r"\bRENDER_TECHNIQUE_(NAIVE|TILED|CLUSTERED)\b").unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumNext)]
#[repr(u32)]
pub enum AttenuationMode {
    Step = 1,
    Linear = 2,
    Physical = 3,
    Interpolated = 4,
    Reduced = 5,
    Smooth = 6,
}

impl AttenuationMode {
    pub fn source(self) -> &'static str {
        match self {
            AttenuationMode::Step => "#define ATTENUATION_MODE_STEP\n",
            AttenuationMode::Linear => "#define ATTENUATION_MODE_LINEAR\n",
            AttenuationMode::Physical => "#define ATTENUATION_MODE_PHYSICAL\n",
            AttenuationMode::Interpolated => "#define ATTENUATION_MODE_INTERPOLATED\n",
            AttenuationMode::Reduced => "#define ATTENUATION_MODE_REDUCED\n",
            AttenuationMode::Smooth => "#define ATTENUATION_MODE_SMOOTH\n",
        }
    }

    pub fn regex() -> Regex {
        Regex::new(r"\bATTENUATION_MODE_(STEP|LINEAR|PHYSICAL|INTERPOLATED|REDUCED|SMOOTH)\b").unwrap()
    }
}

pub struct Shader {
    source_indices: Vec<usize>,
    render_technique: bool,
    attenuation_mode: bool,
    pub branch: ic::Branch,
    name: ShaderName,
}

impl Shader {
    pub fn new(name: ShaderName, source_indices: Vec<usize>) -> Self {
        Self {
            source_indices,
            render_technique: false,
            attenuation_mode: false,
            branch: ic::Branch::dirty(),
            name,
        }
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> ic::Modified {
        let global = &world.global;
        if self.branch.verify(global) {
            let modified = self
                .source_indices
                .iter()
                .map(|&i| world.sources[i].modified)
                .chain(std::iter::once(world.light_space.modified))
                .chain(
                    if self.render_technique {
                        Some(world.render_technique.modified)
                    } else {
                        None
                    }
                    .iter()
                    .copied(),
                )
                .chain(
                    if self.attenuation_mode {
                        Some(world.attenuation_mode.modified)
                    } else {
                        None
                    }
                    .iter()
                    .copied(),
                )
                .max()
                .unwrap_or(ic::Modified::NONE);

            if self.branch.recompute(&modified) {
                let sources: Vec<[String; 2]> = self
                    .source_indices
                    .iter()
                    .map(|&i| [format!("#line 1 {}\n", i + 1), world.sources[i].read()])
                    .collect();

                self.render_technique = sources
                    .iter()
                    .any(|[_, source]| world.render_technique_regex.is_match(source));

                self.attenuation_mode = sources
                    .iter()
                    .any(|[_, source]| world.attenuation_mode_regex.is_match(source));

                self.name.compile(
                    gl,
                    std::iter::once(rendering::COMMON_DECLARATION)
                        .chain(std::iter::once(world.light_space.value.source()))
                        .chain(
                            [
                                rendering::GLOBAL_DATA_DECLARATION,
                                rendering::VIEW_DATA_DECLARATION,
                                rendering::CLS_BUFFER_DECLARATION,
                                rendering::MATERIAL_DATA_DECLARATION,
                            ]
                            .iter()
                            .copied(),
                        )
                        .chain(
                            if self.render_technique {
                                Some(world.render_technique.value.source())
                            } else {
                                None
                            }
                            .iter()
                            .copied(),
                        )
                        .chain(
                            if self.attenuation_mode {
                                Some(world.attenuation_mode.value.source())
                            } else {
                                None
                            }
                            .iter()
                            .copied(),
                        )
                        .chain(sources.iter().flat_map(|x| x.iter().map(|s| s.as_str()))),
                );

                if self.name.is_uncompiled() {
                    let log = self.name.log(gl);

                    let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
                        let i: usize = captures[0].parse().unwrap();
                        if i > 0 {
                            let i = i - 1;
                            let path = world.sources[i].path.strip_prefix(&world.resource_dir).unwrap();
                            path.display().to_string()
                        } else {
                            "<generated header>".to_string()
                        }
                    });

                    error!("Compile error:\n{}", log);
                }
            }
        }

        self.branch.modified()
    }

    pub fn name<'a>(&'a self, global: &'a ic::Global) -> &'a ShaderName {
        self.branch.panic_if_outdated(global);
        &self.name
    }
}

pub struct Program {
    vertex: Shader,
    fragment: Shader,
    branch: ic::Branch,
    name: ProgramName,
}

impl Program {
    pub fn new(gl: &gl::Gl, vertex_source_indices: Vec<usize>, fragment_source_indices: Vec<usize>) -> Self {
        let mut program_name = ProgramName::new(gl);
        let vertex_name = ShaderName::new(gl, gl::VERTEX_SHADER);
        let fragment_name = ShaderName::new(gl, gl::FRAGMENT_SHADER);

        program_name.attach(gl, &[&vertex_name, &fragment_name]);

        Self {
            vertex: Shader::new(vertex_name, vertex_source_indices),
            fragment: Shader::new(fragment_name, fragment_source_indices),
            branch: ic::Branch::dirty(),
            name: program_name,
        }
    }

    pub fn modified(&self) -> ic::Modified {
        self.branch.modified()
    }

    pub fn update(&mut self, gl: &gl::Gl, world: &mut World) -> ic::Modified {
        if self.branch.verify(&world.global) {
            let modified = std::cmp::max(self.vertex.update(gl, world), self.fragment.update(gl, world));

            if self.branch.recompute(&modified) {
                self.name.link(gl);

                if self.name.is_unlinked()
                    && self.vertex.name(&world.global).is_compiled()
                    && self.fragment.name(&world.global).is_compiled()
                {
                    let log = self.name.log(gl);

                    let log = world.gl_log_regex.replace_all(&log, |captures: &regex::Captures| {
                        let i: usize = captures[0].parse().unwrap();
                        if i > 0 {
                            let i = i - 1;
                            let path = world.sources[i].path.strip_prefix(&world.resource_dir).unwrap();
                            path.display().to_string()
                        } else {
                            "<generated header>".to_string()
                        }
                    });

                    error!("Link error:\n{}", log);
                }
            }
        }

        self.branch.modified()
    }

    pub fn name<'a>(&'a self, global: &'a ic::Global) -> &'a ProgramName {
        self.branch.panic_if_outdated(global);
        &self.name
    }
}
