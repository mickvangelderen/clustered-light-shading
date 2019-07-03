#[allow(unused)]
pub(crate) use std::convert::{TryFrom, TryInto};

use bmfont::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut buffer = Vec::new();
    let mut file = File::open("resources/fonts/OpenSans-Regular.fnt").unwrap();
    file.read_to_end(&mut buffer).unwrap();

    let bmfont = BMFont::new(&buffer[..]);
    println!("{:#?}", &bmfont);
}
