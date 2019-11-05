use std::path::PathBuf;
use std::io::Read;

fn main() {
    let dir_path = PathBuf::from(std::env::args().skip(1).next().unwrap());
    for entry in std::fs::read_dir(dir_path).unwrap() {
        let file_path = entry.unwrap().path();
        match file_path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("dds") => {
                let file = std::fs::File::open(&file_path).unwrap();
                let mut reader = std::io::BufReader::new(file);
                let header = dds::RawFileHeader::parse(&mut reader).unwrap();
                if header.mipmap_count.to_ne() == 0 {
                    println!("{:?}", &file_path);
                    println!("flags {:32b}", header.flags.to_ne());
                    println!("caps0 {:32b}", header.caps0.0.to_ne());
                    println!("caps1 {:32b}", header.caps1.0.to_ne());

                    dbg!(&file_path, &header);
                }
            },
            _ => {
                // Ignore.
            }
        }
    }

    return;

}

fn dump_bytes(mut reader: impl Read) {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).unwrap();

    let remainder = bytes.len() % 4;

    for row in 0..((bytes.len() - remainder) / 4) {
        let offset = row * 4;
        let b = [
            bytes[offset + 0],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ];
        let c = [b[0] as char, b[1] as char, b[2] as char, b[3] as char];
        println!(
            "{:08x}: {:08b} {:08b} {:08b} {:08b} = {:4}: [u8; 4] = {:10}: u32le",
            offset,
            b[0],
            b[1],
            b[2],
            b[3],
            format!(
                "{}{}{}{}",
                if c[0].is_ascii_graphic() { c[0] } else { ' ' },
                if c[1].is_ascii_graphic() { c[1] } else { ' ' },
                if c[2].is_ascii_graphic() { c[2] } else { ' ' },
                if c[3].is_ascii_graphic() { c[3] } else { ' ' },
            ),
            u32::from_le_bytes(b),
        );
    }

    if remainder > 0 {
        let offset = bytes.len() - remainder;
        print!("{:08x}: ", offset,);
        for i in 0..remainder {
            print!(" {:08b}", bytes[offset + i]);
        }
        println!();
    }
}
