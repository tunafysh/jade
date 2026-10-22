use super::vertex::Vertex;

#[derive(Debug, Clone)]
pub enum Primitive {
    Points,
    Lines,
    Triangles,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub primitive: Primitive,
}
