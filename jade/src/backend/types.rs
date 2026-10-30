use wgpu::ShaderModule;

use crate::shading::reflection::ShaderReflection;

pub struct ShaderResource {
    pub module: ShaderModule,
    pub reflection: ShaderReflection,
}
