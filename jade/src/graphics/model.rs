use super::{material::Material, mesh::Mesh};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub material_indices: Vec<(u32, u32)>,
}
