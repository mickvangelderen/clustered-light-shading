#![allow(unused_macros)]

macro_rules! get_uniform_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_uniform_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            eprintln!("{}: Could not get uniform location {:?}.", file!(), $s);
        }
        loc
    }};
}

macro_rules! get_attribute_location {
    ($gl: ident, $program: expr, $s: expr) => {{
        let loc = $gl.get_attrib_location($program, gl::static_cstr!($s));
        if loc.is_none() {
            eprintln!("{}: Could not get attribute location {:?}.", file!(), $s);
        }
        loc
    }};
}

