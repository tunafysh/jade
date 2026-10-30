use std::collections::HashMap;

use naga::{AddressSpace, Binding, TypeInner};

use crate::shading::shader::ShaderStage;

#[derive(Debug)]
pub struct ShaderEntryPoint {
    pub name: String,
    pub inputs: Vec<ShaderField>,
    pub stage: ShaderStage,
}

#[derive(Debug)]
pub struct ShaderReflection {
    pub resources: HashMap<String, ShaderResource>,
    pub blocks: HashMap<String, ShaderBlock>,
    pub entry_points: Vec<ShaderEntryPoint>,
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
    pub address_space: AddressSpace,
    pub size: u32,
    pub fields: Vec<ShaderField>,
}

#[derive(Debug)]
pub struct ShaderField {
    pub name: Option<String>,
    pub ty: TypeInner,
    pub binding: Option<Binding>,
}
