use glam::{Quat, Vec3};

#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
