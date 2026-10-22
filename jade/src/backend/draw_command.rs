use crate::{
    graphics::{material::Material, mesh::Mesh},
    node::transform::Transform,
    shading::shader::Shader,
};

#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub mesh: Mesh,
    pub material: Material,
    pub shader: Shader,
    pub transform: Transform,
}
