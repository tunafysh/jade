use anyhow::Result;
use glam::{Vec2, Vec3};
use std::path::Path;

use crate::graphics::{
    mesh::{Mesh, Primitive},
    vertex::Vertex,
};

pub struct ObjImporter;

impl ObjImporter {
    pub fn import(path: &Path) -> Result<Vec<Mesh>> {
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        models
            .into_iter()
            .map(|model| Self::build_mesh(model.mesh))
            .collect()
    }

    fn build_mesh(mesh: tobj::Mesh) -> Result<Mesh> {
        let vertices = mesh
            .positions
            .chunks_exact(3)
            .enumerate()
            .map(|(index, position)| Vertex {
                position: Vec3::from_array([position[0], position[1], position[2]]),
                normal: Self::vec3_at(&mesh.normals, index),
                uv: Self::vec2_at(&mesh.texcoords, index),
                color: Self::vec3_at(&mesh.vertex_color, index),
            })
            .collect();

        Ok(Mesh {
            vertices,
            indices: mesh.indices,
            primitive: Primitive::Triangles,
        })
    }

    fn vec2_at(values: &[f32], index: usize) -> Vec2 {
        values
            .get(index * 2..index * 2 + 2)
            .map(|values| Vec2::from_array([values[0], values[1]]))
            .unwrap_or(Vec2::ZERO)
    }

    fn vec3_at(values: &[f32], index: usize) -> Vec3 {
        values
            .get(index * 3..index * 3 + 3)
            .map(|values| Vec3::from_array([values[0], values[1], values[2]]))
            .unwrap_or(Vec3::ZERO)
    }
}
