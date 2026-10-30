use super::vertex::Vertex;

#[derive(Debug, Clone, Default)]
pub enum Primitive {
    Points,
    Lines,
    #[default]
    Triangles,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub primitive: Primitive,
}
