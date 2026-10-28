use anyhow::Result;
use ash::{
    Device,
    util::read_spv,
    vk::{Handle, ShaderModule, ShaderModuleCreateInfo},
};
use std::io::Cursor;
use tracing::{info, instrument};

#[instrument(name = "Vulkan - Create Shader Module", skip_all)]
pub fn create_shader_module(device: &Device, spirv: &[u8]) -> Result<ShaderModule> {
    let mut cursor = Cursor::new(spirv);
    let shader_bytecode = read_spv(&mut cursor)?;
    info!(
        byte_count = spirv.len(),
        word_count = shader_bytecode.len(),
        "Parsed Vulkan SPIR-V shader"
    );
    let shader_info = ShaderModuleCreateInfo::default().code(&shader_bytecode);
    let module = unsafe { device.create_shader_module(&shader_info, None) }?;
    info!(
        shader_module = module.as_raw(),
        "Vulkan shader module created"
    );
    Ok(module)
}
