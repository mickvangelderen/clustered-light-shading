use std::path::PathBuf;

fn main() {
    let file_path = PathBuf::from(std::env::args().skip(1).next().unwrap());
    let bytes = std::fs::read(&file_path).unwrap();

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
