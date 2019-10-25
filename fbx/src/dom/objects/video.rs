use crate::tree::*;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Video {
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub properties: VideoProperties,
}

#[derive(Debug)]
pub struct VideoProperties {}

impl Default for VideoProperties {
    fn default() -> Self {
        Self {}
    }
}

impl Video {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let id = node.properties[0].to_i64_exact() as u64;

        let name = {
            let name = node.properties[1].as_str();
            let postfix = "\u{0}\u{1}Video";
            assert!(name.ends_with(postfix));
            String::from(&name[0..name.len() - postfix.len()])
        };

        assert_eq!("Clip", node.properties[2].as_str());

        let mut kind = None;
        let mut file_path = None;
        let properties = VideoProperties::default();

        for node in node.children.iter() {
            stack.push(node.name.clone());
            match node.name.as_str() {
                "UseMipMap" | "Filename" => {
                    // Don't care.
                }
                "Type" => {
                    assert!(kind.is_none());
                    kind = Some(node.properties[0].as_str().to_string());
                }
                "RelativeFilename" => {
                    assert!(file_path.is_none());
                    file_path = Some(PathBuf::from(node.properties[0].as_str()));
                }
                "Properties70" => {
                    for node in node.children.iter() {
                        stack.push(node.name.clone());

                        assert_eq!(node.name.as_str(), "P");

                        match node.properties[0].as_str() {
                            "Path" | "RelPath" | "Color" | "ClipIn" | "ClipOut" | "Mute" => {
                                // Don't care.
                            }
                            unknown => {
                                panic!("Unknown video properties property: {:?}", unknown);
                            }
                        }

                        stack.pop();
                    }
                }
                unknown => {
                    panic!("Unknown video property: {:?}", unknown);
                }
            }
            stack.pop();
        }

        stack.pop();

        Video {
            id,
            name,
            kind: kind.unwrap(),
            file_path: file_path.unwrap(),
            properties,
        }
    }
}
