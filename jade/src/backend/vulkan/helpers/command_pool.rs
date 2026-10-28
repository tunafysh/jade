use anyhow::Result;
use ash::{
    Device,
    vk::{
        CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPool,
        CommandPoolCreateFlags, CommandPoolCreateInfo, Handle,
    },
};
use tracing::{info, instrument};

#[instrument(name = "Vulkan - Create Command Pool", skip_all)]
pub fn create_command_pool(
    logical_device: &Device,
    graphics_family_index: u32,
) -> Result<CommandPool> {
    let command_pool_info = CommandPoolCreateInfo::default()
        .queue_family_index(graphics_family_index)
        .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = unsafe { logical_device.create_command_pool(&command_pool_info, None) }?;
    info!(
        command_pool = command_pool.as_raw(),
        graphics_family_index, "Vulkan command pool created"
    );

    Ok(command_pool)
}

#[instrument(name = "Vulkan - Allocate Command Buffers", skip_all)]
pub fn allocate_command_buffers(
    logical_device: &Device,
    command_pool: CommandPool,
) -> Result<Vec<CommandBuffer>> {
    let creation_info = CommandBufferAllocateInfo::default()
        .command_buffer_count(1)
        .command_pool(command_pool)
        .level(CommandBufferLevel::PRIMARY);

    let buffers = unsafe { logical_device.allocate_command_buffers(&creation_info)? };
    info!(
        command_pool = command_pool.as_raw(),
        count = buffers.len(),
        "Vulkan command buffers allocated"
    );

    Ok(buffers)
}
