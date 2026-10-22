use std::collections::HashMap;

#[derive(Debug)]
pub struct ShaderReflection {
    pub resources: HashMap<String, ShaderResource>,
    pub blocks: HashMap<String, ShaderBlock>,
}

#[derive(Debug)]
pub struct ShaderResource {
    pub group: u32,
    pub binding: u32,
    pub kind: ShaderResourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderResourceKind {
    UniformBuffer,
    StorageBuffer,
    Texture,
    Sampler,
}

#[derive(Debug)]
pub struct ShaderBlock {
    pub address_space: ShaderAddressSpace,
    pub size: u32,
    pub fields: Vec<ShaderField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderAddressSpace {
    Uniform,
    Storage,
}

#[derive(Debug)]
pub struct ShaderField {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub ty: ShaderFieldType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderFieldType {
    Float,
    Int,
    Uint,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
}
