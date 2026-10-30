use crate::{
    graphics::{material::Material, mesh::Mesh},
    node::transform::Transform,
};

#[derive(Clone, Debug, Default)]
pub struct DrawCommand {
    pub mesh: Mesh,
    pub material: Material,
    pub shader: String,
    pub transform: Transform,
}

impl DrawCommand {
    pub fn shader(&mut self, name: String) {
        self.shader = name;
    }
}
