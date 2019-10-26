use crate::{dom::*, tree::*};

pub struct Root {
    pub global_settings: GlobalSettings,
    pub objects: Objects,
    pub connections: TypedConnections,
}

impl Root {
    pub fn from_fbx_file(file: &File) -> Self {
        let stack = &mut Vec::new();

        let mut global_settings: Option<GlobalSettings> = None;
        let mut objects: Option<Objects> = None;
        let mut connections: Option<Connections> = None;

        for node in file.children.iter() {
            stack.push(node.name.to_string());

            match node.name.as_str() {
                "GlobalSettings" => {
                    assert!(global_settings.is_none());
                    global_settings = Some(GlobalSettings::from_fbx(node, stack));
                }
                "Objects" => {
                    assert!(objects.is_none());
                    objects = Some(Objects::from_fbx(node, stack));
                }
                "Connections" => {
                    assert!(connections.is_none());
                    connections = Some(Connections::from_fbx(node, stack));
                }
                _ => {
                    // Don't care.
                }
            }

            stack.pop();
        }

        let objects = objects.unwrap();
        let untyped_connections = connections.unwrap();
        let connections = TypedConnections::new(&objects, &untyped_connections);

        Self {
            objects,
            global_settings: global_settings.unwrap(),
            connections,
        }
    }
}
