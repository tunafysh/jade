use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};

#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: Vec3,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct CVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

impl From<Vertex> for CVertex {
    fn from(v: Vertex) -> Self {
        Self {
            position: v.position.into(),
            normal: v.normal.into(),
            uv: v.uv.into(),
            color: v.color.into(),
        }
    }
}

impl From<CVertex> for Vertex {
    fn from(v: CVertex) -> Self {
        Self {
            position: v.position.into(),
            normal: v.normal.into(),
            uv: v.uv.into(),
            color: v.color.into(),
        }
    }
}

impl From<&Vertex> for CVertex {
    fn from(v: &Vertex) -> Self {
        Self {
            position: v.position.into(),
            normal: v.normal.into(),
            uv: v.uv.into(),
            color: v.color.into(),
        }
    }
}

impl From<&CVertex> for Vertex {
    fn from(v: &CVertex) -> Self {
        Self {
            position: v.position.into(),
            normal: v.normal.into(),
            uv: v.uv.into(),
            color: v.color.into(),
        }
    }
}
