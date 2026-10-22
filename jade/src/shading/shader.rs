use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderValue {
    Float(f32),
    Int(i32),
    Uint(u32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone)]
pub enum ShaderCode {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct ShaderOutput {
    pub code: HashMap<ShaderStage, ShaderCode>,
    pub entry_points: HashMap<ShaderStage, String>,
}

#[derive(Debug, Clone)]
pub struct Shader {
    pub output: ShaderOutput,
}
