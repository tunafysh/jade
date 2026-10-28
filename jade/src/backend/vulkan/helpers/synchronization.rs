use anyhow::Result;
use ash::{
    Device,
    vk::{Fence, FenceCreateFlags, FenceCreateInfo, Handle, Semaphore, SemaphoreCreateInfo},
};
use tracing::{info, instrument};

#[instrument(name = "Vulkan - Create Semaphore", skip_all)]
pub fn create_semaphore(logical_device: &Device) -> Result<Semaphore> {
    let creation_info = SemaphoreCreateInfo::default();

    let result = unsafe { logical_device.create_semaphore(&creation_info, None) }?;
    info!(semaphore = result.as_raw(), "Vulkan semaphore created");
    Ok(result)
}

#[instrument(name = "Vulkan - Create Signaled Fence", skip_all)]
pub fn create_signaled_fence(logical_device: &Device) -> Result<Fence> {
    let creation_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);

    let result = unsafe { logical_device.create_fence(&creation_info, None) }?;
    info!(
        fence = result.as_raw(),
        signaled = true,
        "Vulkan fence created"
    );

    Ok(result)
}
