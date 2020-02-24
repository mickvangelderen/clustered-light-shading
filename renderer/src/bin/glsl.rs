pub trait WriteGlsl {
    fn write_glsl<W>(&self, w: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write;
}

pub trait ToGlsl {
    fn to_glsl(&self) -> String;
}

impl<T: WriteGlsl + ?Sized> ToGlsl for T {
    #[inline]
    fn to_glsl(&self) -> String {
        let mut output = String::new();
        self.write_glsl(&mut output)
            .expect("a WriteGlsl trait implementation returned an error unexpectedly");
        output
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Pass {
    InitialSum,
    OffsetSum,
    FinalSum,
}

impl WriteGlsl for Pass {
    fn write_glsl<W>(&self, w: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        w.write_str(
            "\
#define PASS_INITIAL_SUM 1\n\
#define PASS_OFFSET_SUM 2\n\
#define PASS_FINAL_SUM 3\n\
#define PASS ",
        )?;
        w.write_str(match self {
            Self::InitialSum => "1\n",
            Self::OffsetSum => "2\n",
            Self::FinalSum => "3\n",
        })?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Configuration {
    pub pass: Pass,
    pub wide_thread_count: u32,
    pub narrow_thread_count: u32,
}

impl WriteGlsl for Configuration {
    fn write_glsl<W>(&self, w: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        self.pass.write_glsl(w)?;
        w.write_fmt(format_args!(
            "\
#define WIDE_THREAD_COUNT {}\n\
#define NARROW_THREAD_COUNT {}\n\
",
            self.wide_thread_count, self.narrow_thread_count
        ))?;
        Ok(())
    }
}

#[derive(Debug)]
struct Program {
    configuration: Configuration,
    name: u32,
}

struct Gl {
    programs: Vec<bool>,
}

impl Gl {
    pub fn new() -> Self {
        Self {
            programs: std::iter::repeat(false).take(3).collect(),
        }
    }

    pub fn create_program(&mut self) -> Option<u32> {
        for (i, p) in self.programs.iter_mut().enumerate() {
            if *p == false {
                *p = true;
                return Some(i as u32 + 1);
            }
        }
        return None;
    }

    pub fn delete_program(&mut self, p: u32) {
        let i = (p - 1) as usize;
        assert_eq!(true, self.programs[i], "double free");
        self.programs[i] = false;
    }
}

impl Program {
    pub fn new(gl: &mut Gl, configuration: Configuration) -> Self {
        Self {
            configuration,
            name: gl.create_program().unwrap(),
        }
    }

    pub fn reconcile(&mut self, gl: &mut Gl, configuration: Configuration) {
        if self.configuration == configuration {
            // We're good to go as is.
            return;
        }
        unsafe {
            std::ptr::read(self).drop(gl);
            std::ptr::write(self, Program::new(gl, configuration));
        }
    }

    pub fn drop(self, gl: &mut Gl) {
        gl.delete_program(self.name);
    }
}

fn main() {
    let gl = &mut Gl::new();

    let mut p0 = Program::new(
        gl,
        Configuration {
            pass: Pass::InitialSum,
            wide_thread_count: 128,
            narrow_thread_count: 480,
        },
    );

    let mut p1 = Program::new(
        gl,
        Configuration {
            pass: Pass::OffsetSum,
            wide_thread_count: 128,
            narrow_thread_count: 480,
        },
    );

    let mut p2 = Program::new(
        gl,
        Configuration {
            pass: Pass::FinalSum,
            wide_thread_count: 128,
            narrow_thread_count: 480,
        },
    );

    dbg!(&p0);

    for p in [&mut p0, &mut p1, &mut p2].iter_mut() {
        p.reconcile(
            gl,
            Configuration {
                wide_thread_count: 200,
                ..p.configuration
            },
        );
    }

    dbg!(&p0);
}
