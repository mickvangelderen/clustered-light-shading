use crate::dom::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypedConnections {
    pub oo: Vec<(TypedIndex, TypedIndex)>,
    pub op: Vec<(TypedIndex, TypedIndex, String)>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TypedIndex {
    Root,
    Geometry(usize),
    Model(usize),
    Material(usize),
    Texture(usize),
    Video(usize),
    Unknown(i64),
}

impl TypedConnections {
    pub fn new(objects: &Objects, connections: &Connections) -> Self {
        let geometry_map: HashMap<i64, usize> = objects.geometries.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        let model_map: HashMap<i64, usize> = objects.models.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        let material_map: HashMap<i64, usize> = objects.materials.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        let texture_map: HashMap<i64, usize> = objects.textures.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
        let video_map: HashMap<i64, usize> = objects.videos.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

        let lookup = |id: i64| {
            if id == 0 {
                return TypedIndex::Root;
            }
            if let Some(&index) = geometry_map.get(&id) {
                return TypedIndex::Geometry(index);
            }
            if let Some(&index) = model_map.get(&id) {
                return TypedIndex::Model(index);
            }
            if let Some(&index) = material_map.get(&id) {
                return TypedIndex::Material(index);
            }
            if let Some(&index) = texture_map.get(&id) {
                return TypedIndex::Texture(index);
            }
            if let Some(&index) = video_map.get(&id) {
                return TypedIndex::Video(index);
            }
            return TypedIndex::Unknown(id);
        };

        Self {
            oo: connections.oo.iter().map(|oo| (lookup(oo.0), lookup(oo.1))).collect(),
            op: connections
                .op
                .iter()
                .map(|op| (lookup(op.0), lookup(op.1), op.2.clone()))
                .collect(),
        }
    }
}
