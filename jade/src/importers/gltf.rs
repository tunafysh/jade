use anyhow::{Result, anyhow};
use std::path::Path;

use crate::graphics::{
    mesh::{Mesh, Primitive},
    model::Model,
    vertex::Vertex,
};

pub struct GltfImporter;

impl GltfImporter {
    pub fn import(path: &Path) -> Result<Model> {
        let (document, buffers, _images) = gltf::import(path)?;

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = reader
                    .read_positions()
                    .ok_or(anyhow!("Primitive has no positions."))?;

                let normals = reader
                    .read_normals()
                    .ok_or(anyhow!("Primitive has no normals."))?;

                let uvs = reader
                    .read_tex_coords(0)
                    .ok_or(anyhow!("Primitive has no texture coordinates."))?
                    .into_f32();

                let colors = reader
                    .read_colors(0)
                    .ok_or(anyhow!("Primitive has no vertex colors."))?
                    .into_rgba_f32();

                let indices = reader
                    .read_indices()
                    .ok_or_else(|| anyhow!("Primitive has no indices."))?
                    .into_u32()
                    .collect::<Vec<u32>>();

                let vertices = positions
                    .zip(normals)
                    .zip(uvs)
                    .zip(colors)
                    .map(|(((position, normal), uv), color)| Vertex {
                        position: position.into(),
                        normal: normal.into(),
                        uv: uv.into(),
                        color: color.into(),
                    })
                    .collect::<Vec<_>>();

                meshes.push(Mesh {
                    vertices,
                    indices,
                    primitive: Primitive::Triangles,
                });
            }
        }

        Ok(Model {
            meshes,
            materials: Vec::new(),
            material_indices: Vec::new(),
        })
    }
}
