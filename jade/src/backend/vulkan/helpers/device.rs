use crate::backend::vulkan::types::QueueFamilyIndices;
use anyhow::{Result, anyhow};
use ash::{
    Device, Instance,
    khr::surface::Instance as SurfaceInstance,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, Handle, PhysicalDevice,
        PhysicalDeviceVulkan13Features, QueueFlags, SurfaceKHR,
    },
};
use tracing::{info, instrument};

#[instrument(name = "Vulkan - Find Queue Families", skip_all)]
pub fn find_queue_families(
    instance: &Instance,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
    device: PhysicalDevice,
) -> Result<QueueFamilyIndices> {
    let families = unsafe { instance.get_physical_device_queue_family_properties(device) };
    info!(
        device = device.as_raw(),
        family_count = families.len(),
        "Inspecting Vulkan queue families"
    );
    let mut indices = QueueFamilyIndices {
        graphics: None,
        present: None,
        compute: None,
        transfer: None,
    };
    for (index, family) in families.iter().enumerate() {
        let index = index as u32;
        if family.queue_flags.contains(QueueFlags::GRAPHICS) {
            indices.graphics = Some(index);
        }
        if unsafe { surface_loader.get_physical_device_surface_support(device, index, surface)? } {
            indices.present = Some(index);
        }
        if family.queue_flags.contains(QueueFlags::COMPUTE) {
            indices.compute = Some(index);
        }
        if family.queue_flags.contains(QueueFlags::TRANSFER) {
            indices.transfer = Some(index);
        }
    }
    info!(graphics = ?indices.graphics, present = ?indices.present, compute = ?indices.compute, transfer = ?indices.transfer, "Vulkan queue families selected");
    Ok(indices)
}

#[instrument(name = "Vulkan - Create Logical Device", skip_all)]
pub fn create_logical_device(
    instance: &mut Instance,
    physical_device: PhysicalDevice,
    surface_loader: &SurfaceInstance,
    surface: &mut SurfaceKHR,
) -> Result<(Device, QueueFamilyIndices)> {
    let queue_families = find_queue_families(instance, surface_loader, *surface, physical_device)?;
    let graphics_family = queue_families
        .graphics
        .ok_or_else(|| anyhow!("No graphics queue family"))?;
    info!(device = physical_device.as_raw(), graphics_family, present_family = ?queue_families.present, "Creating Vulkan logical device");
    let queue_priority = [1.0];
    let queue_create_info = [DeviceQueueCreateInfo::default()
        .queue_family_index(graphics_family)
        .queue_priorities(&queue_priority)];
    let mut dynamic_rendering = PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);
    let extensions = [ash::khr::swapchain::NAME.as_ptr()];
    let device_create_info = DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_info)
        .enabled_extension_names(&extensions)
        .push_next(&mut dynamic_rendering);
    let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }?;
    info!(
        device = physical_device.as_raw(),
        dynamic_rendering = true,
        "Vulkan logical device created"
    );
    Ok((device, queue_families))
}
