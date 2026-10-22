use anyhow::{Result, anyhow};
use glam::{Vec2, Vec3};
use std::{collections::HashMap, fs::read_to_string, path::Path};

use crate::graphics::{
    mesh::{Mesh, Primitive},
    vertex::Vertex,
};

pub struct ObjImporter;

impl ObjImporter {
    pub fn import(path: &Path) -> Result<Vec<Mesh>> {
        let source = read_to_string(path)?;

        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();

        for line in source.lines() {
            let mut parts = line.split_whitespace();

            let Some(kind) = parts.next() else {
                continue;
            };

            match kind {
                "v" => positions.push(Self::parse_position(&mut parts)?),
                "vt" => uvs.push(Self::parse_uv(&mut parts)?),
                "vn" => normals.push(Self::parse_normal(&mut parts)?),
                "f" => faces.push(Self::parse_face(&mut parts)?),
                _ => {}
            }
        }

        Ok(vec![Self::build_mesh(positions, uvs, normals, faces)?])
    }

    fn parse_position<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Vec3> {
        Ok(Vec3::new(
            Self::parse_f32(parts.next(), "position X")?,
            Self::parse_f32(parts.next(), "position Y")?,
            Self::parse_f32(parts.next(), "position Z")?,
        ))
    }

    fn parse_uv<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Vec2> {
        Ok(Vec2::new(
            Self::parse_f32(parts.next(), "UV U")?,
            Self::parse_f32(parts.next(), "UV V")?,
        ))
    }

    fn parse_normal<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Vec3> {
        Ok(Vec3::new(
            Self::parse_f32(parts.next(), "normal X")?,
            Self::parse_f32(parts.next(), "normal Y")?,
            Self::parse_f32(parts.next(), "normal Z")?,
        ))
    }

    fn parse_f32(value: Option<&str>, name: &str) -> Result<f32> {
        value
            .ok_or_else(|| anyhow!("Missing {}.", name))?
            .parse::<f32>()
            .map_err(Into::into)
    }

    fn parse_vertex(value: &str) -> Result<(u32, u32, u32)> {
        let mut parts = value.split('/');

        let position = Self::parse_index(parts.next(), "position")?;
        let uv = Self::parse_index(parts.next(), "UV")?;
        let normal = Self::parse_index(parts.next(), "normal")?;

        Ok((position, uv, normal))
    }

    fn parse_index(value: Option<&str>, name: &str) -> Result<u32> {
        let index = value
            .ok_or_else(|| anyhow!("Missing {} index.", name))?
            .parse::<u32>()?;

        index
            .checked_sub(1)
            .ok_or_else(|| anyhow!("Invalid {} index: 0.", name))
    }

    fn parse_face<'a>(
        parts: &mut impl Iterator<Item = &'a str>,
    ) -> Result<((u32, u32, u32), (u32, u32, u32), (u32, u32, u32))> {
        let a = Self::parse_vertex(
            parts
                .next()
                .ok_or_else(|| anyhow!("Missing first face vertex."))?,
        )?;

        let b = Self::parse_vertex(
            parts
                .next()
                .ok_or_else(|| anyhow!("Missing second face vertex."))?,
        )?;

        let c = Self::parse_vertex(
            parts
                .next()
                .ok_or_else(|| anyhow!("Missing third face vertex."))?,
        )?;

        Ok((a, b, c))
    }

    fn build_mesh(
        positions: Vec<Vec3>,
        uvs: Vec<Vec2>,
        normals: Vec<Vec3>,
        faces: Vec<((u32, u32, u32), (u32, u32, u32), (u32, u32, u32))>,
    ) -> Result<Mesh> {
        let mut vertex_map = HashMap::<(u32, u32, u32), u32>::new();
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();

        for (a, b, c) in faces {
            let a = Self::resolve_vertex(
                a,
                &positions,
                &uvs,
                &normals,
                &mut vertex_map,
                &mut vertices,
            )?;

            let b = Self::resolve_vertex(
                b,
                &positions,
                &uvs,
                &normals,
                &mut vertex_map,
                &mut vertices,
            )?;

            let c = Self::resolve_vertex(
                c,
                &positions,
                &uvs,
                &normals,
                &mut vertex_map,
                &mut vertices,
            )?;

            indices.extend([a, b, c]);
        }

        Ok(Mesh {
            vertices,
            indices,
            primitive: Primitive::Triangles,
        })
    }

    fn resolve_vertex(
        key: (u32, u32, u32),
        positions: &[Vec3],
        uvs: &[Vec2],
        normals: &[Vec3],
        vertex_map: &mut HashMap<(u32, u32, u32), u32>,
        vertices: &mut Vec<Vertex>,
    ) -> Result<u32> {
        if let Some(&index) = vertex_map.get(&key) {
            return Ok(index);
        }

        let position = *positions
            .get(key.0 as usize)
            .ok_or_else(|| anyhow!("Position index {} out of bounds.", key.0 + 1))?;

        let uv = *uvs
            .get(key.1 as usize)
            .ok_or_else(|| anyhow!("UV index {} out of bounds.", key.1 + 1))?;

        let normal = *normals
            .get(key.2 as usize)
            .ok_or_else(|| anyhow!("Normal index {} out of bounds.", key.2 + 1))?;

        let index = vertices.len() as u32;

        vertices.push(Vertex {
            position,
            uv,
            normal,
        });

        vertex_map.insert(key, index);

        Ok(index)
    }
}
