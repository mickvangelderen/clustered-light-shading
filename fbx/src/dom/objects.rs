mod geometry;
mod material;
mod model;
mod texture;
mod video;

pub use geometry::*;
pub use material::*;
pub use model::*;
pub use texture::*;
pub use video::*;

use crate::tree::*;

#[derive(Debug)]
pub struct Objects {
    pub geometries: Vec<Geometry>,
    pub materials: Vec<Material>,
    pub models: Vec<Model>,
    pub textures: Vec<Texture>,
    pub videos: Vec<Video>,
}

impl Objects {
    pub fn from_fbx(node: &Node, stack: &mut Vec<String>) -> Self {
        stack.push(node.name.clone());

        let mut geometries = Vec::new();
        let mut materials = Vec::new();
        let mut models = Vec::new();
        let mut textures = Vec::new();
        let mut videos = Vec::new();

        for child in node.children.iter() {
            match child.name.as_str() {
                "Geometry" => {
                    geometries.push(Geometry::from_fbx(child, stack));
                }
                "Material" => {
                    materials.push(Material::from_fbx(child, stack));
                }
                "Model" => {
                    models.push(Model::from_fbx(child, stack));
                }
                "Texture" => {
                    textures.push(Texture::from_fbx(child, stack));
                }
                "Video" => {
                    videos.push(Video::from_fbx(child, stack));
                }
                _ => {
                    // Ignore.
                }
            }
        }

        stack.pop();

        Self {
            geometries,
            materials,
            models,
            textures,
            videos,
        }
    }
}
