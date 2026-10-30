use glam::Vec4;

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub color: Vec4,
    pub roughness: f32,
    pub metallic: f32,
}
