use crate::*;

#[derive(Debug)]
pub struct BMFont<'a> {
    pub info: &'a InfoBlock,
    pub font_name: &'a CStr,
    pub common: &'a CommonBlock,
    pub pages: Vec<&'a CStr>,
    pub chars: &'a [CharBlock],
    pub kerning_pairs: Option<&'a [KerningPairBlock]>,
}

impl<'a> BMFont<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let mut input = Input::new(bytes);

        let file_header = input.read_raw::<FileHeader>().unwrap();
        assert_eq!(file_header.magic, [66, 77, 70]);
        assert_eq!(file_header.version, 3);

        let mut info_font_name: Option<(&'a InfoBlock, &'a CStr)> = None;
        let mut common: Option<&'a CommonBlock> = None;
        let mut pages: Vec<&'a CStr> = Vec::new();
        let mut chars: Option<&'a [CharBlock]> = None;
        let mut kerning_pairs: Option<&'a [KerningPairBlock]> = None;

        while let Some(block_header) = input.read_raw::<BlockHeader>() {
            let block_kind = block_header.kind.try_into();
            let block_byte_size = block_header.byte_size.to_ne() as usize;

            match block_kind {
                Ok(BlockKind::Info) => {
                    if info_font_name.is_some() {
                        panic!("Cannot handle more than one info block!");
                    }
                    let block = input.read_raw::<InfoBlock>().unwrap();
                    let font_name_len = block_byte_size - std::mem::size_of::<InfoBlock>();
                    let font_name = std::ffi::CStr::from_bytes_with_nul(input.read_bytes(font_name_len).unwrap()).unwrap();
                    info_font_name = Some((block, font_name));
                }
                Ok(BlockKind::Common) => {
                    if common.is_some() {
                        panic!("Cannot handle more than one common block!");
                    }
                    let block = input.read_raw::<CommonBlock>().unwrap();
                    common = Some(block);
                }
                Ok(BlockKind::Pages) => {
                    if pages.len() != 0 {
                        panic!("Cannot handle more than one page block!");
                    }
                    let page_name_len = input.bytes().iter().position(|&b| b == b'\0').unwrap() + 1;
                    assert_eq!(0, block_byte_size % page_name_len);
                    let page_name_count = block_byte_size / page_name_len;

                    pages.reserve(page_name_count);

                    for _ in 0..page_name_count {
                        let page_name =
                            std::ffi::CStr::from_bytes_with_nul(input.read_bytes(page_name_len).unwrap()).unwrap();
                        pages.push(page_name);
                    }
                }
                Ok(BlockKind::Chars) => {
                    if chars.is_some() {
                        panic!("Cannot handle more than one char block!");
                    }
                    assert_eq!(0, block_byte_size % std::mem::size_of::<CharBlock>());
                    let char_block_count = block_byte_size / std::mem::size_of::<CharBlock>();
                    let char_blocks = input.read_raw_array::<CharBlock>(char_block_count).unwrap();
                    chars = Some(char_blocks);
                }
                Ok(BlockKind::KerningPairs) => {
                    if kerning_pairs.is_some() {
                        panic!("Cannot handle more than one kerning pair block!");
                    }
                    assert_eq!(0, block_byte_size % std::mem::size_of::<KerningPairBlock>());
                    let kerning_pair_block_count = block_byte_size / std::mem::size_of::<KerningPairBlock>();
                    let kerning_pair_blocks = input.read_raw_array::<KerningPairBlock>(kerning_pair_block_count).unwrap();
                    kerning_pairs = Some(kerning_pair_blocks);
                }
                Err(err) => {
                    panic!("Unknown block kind: {:?}", err);
                }
            }
        }

        let (info, font_name) = info_font_name.expect("Exactly one info block!");
        Self {
            info,
            font_name,
            common: common.expect("Exactly one common block!"),
            pages,
            chars: chars.expect("Exactly one char block!"),
            kerning_pairs,
        }
    }
}
