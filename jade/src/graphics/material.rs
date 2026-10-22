use glam::Vec4;

#[derive(Debug, Clone)]
pub struct Material {
    pub color: Vec4,
    pub roughness: f32,
    pub metallic: f32,
}
