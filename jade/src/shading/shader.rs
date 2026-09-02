use std::{collections::HashMap, fs::read_to_string, path::Path};


pub enum ShaderParameter {
    Float(f32),
    Int(i32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([[f32; 4]; 4]),
}

pub enum ShaderOutput {
    SpirV(Vec<u32>),
    Glsl(String),
    Wgsl(String),
}
pub struct Shader {
    output: Option<ShaderOutput>,
    parameters: HashMap<String, ShaderParameter>,
    source: String
}

impl Shader {
    pub fn new(path: &Path) -> Result<Self>{

        let source = read_to_string(path)?;

        Ok(Self{
            output: None,
            parameters: HashMap::new(),
            source
        })     
    }
}