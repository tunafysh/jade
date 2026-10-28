use crate::backend::{vulkan::helpers::device::find_queue_families, window::Window};
use anyhow::{Result, anyhow, bail};
use ash::{
    Entry, Instance,
    khr::surface::Instance as SurfaceInstance,
    vk::{
        Handle, MemoryHeapFlags, PhysicalDevice, PhysicalDeviceType, SurfaceKHR, make_api_version,
    },
};
use std::ffi::CStr;
use tracing::{info, instrument};
use winit::raw_window_handle::HasDisplayHandle;

const REQUIRED_INSTANCE_EXTENSIONS: [&CStr; 1] = [c"VK_EXT_swapchain_colorspace"];

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";
const SWAPCHAIN_EXTENSION: &CStr = c"VK_KHR_swapchain";
const REQUIRED_API_VERSION: u32 = make_api_version(0, 1, 3, 0);

#[instrument(name = "Vulkan - Create Instance", skip_all)]
pub fn create_instance(window: &Window) -> Result<(Entry, Instance)> {
    info!("Loading Vulkan loader");
    let entry = unsafe { Entry::load() }?;

    let display_handle = window.handle().display_handle()?;

    let mut extension_names =
        ash_window::enumerate_required_extensions(display_handle.as_raw())?.to_vec();

    let available_extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };
    info!(
        available_extension_count = available_extensions.len(),
        required_extension_count = REQUIRED_INSTANCE_EXTENSIONS.len(),
        "Vulkan instance extensions enumerated"
    );

    for required in REQUIRED_INSTANCE_EXTENSIONS {
        let available = available_extensions.iter().any(|extension| unsafe {
            CStr::from_ptr(extension.extension_name.as_ptr()) == required
        });

        if !available {
            bail!(
                "Required Vulkan instance extension unavailable: {}",
                required.to_string_lossy()
            );
        }

        extension_names.push(required.as_ptr());
    }

    let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };

    let enable_validation = available_layers
        .iter()
        .any(|layer| unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) == VALIDATION_LAYER });
    info!(
        validation = enable_validation,
        extension_count = extension_names.len(),
        "Vulkan instance configuration selected"
    );

    let validation_layers = [VALIDATION_LAYER.as_ptr()];

    let app_info = ash::vk::ApplicationInfo {
        api_version: REQUIRED_API_VERSION,
        p_application_name: c"JADE".as_ptr(),
        p_engine_name: c"JADE".as_ptr(),
        ..Default::default()
    };

    let create_info = ash::vk::InstanceCreateInfo {
        p_application_info: &app_info,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
        pp_enabled_layer_names: if enable_validation {
            validation_layers.as_ptr()
        } else {
            std::ptr::null()
        },
        enabled_layer_count: u32::from(enable_validation),
        ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&create_info, None) }?;
    info!(
        api_version = REQUIRED_API_VERSION,
        "Vulkan instance created"
    );

    Ok((entry, instance))
}

#[instrument(name = "Vulkan - Check Physical Device", skip_all)]
fn is_device_suitable(
    instance: &Instance,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
    device: PhysicalDevice,
) -> Result<bool> {
    let properties = unsafe { instance.get_physical_device_properties(device) };
    let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) };
    info!(device = device.as_raw(), name = %device_name.to_string_lossy(), api_version = properties.api_version, device_type = properties.device_type.as_raw(), "Evaluating Vulkan physical device");

    // Reject devices that cannot support the engine's minimum Vulkan version.
    if properties.api_version < REQUIRED_API_VERSION {
        info!(
            device = device.as_raw(),
            "Rejecting Vulkan device: API version too old"
        );
        return Ok(false);
    }

    // JADE requires the swapchain device extension.
    let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };

    let has_swapchain = extensions.iter().any(|extension| unsafe {
        CStr::from_ptr(extension.extension_name.as_ptr()) == SWAPCHAIN_EXTENSION
    });

    if !has_swapchain {
        info!(
            device = device.as_raw(),
            "Rejecting Vulkan device: swapchain extension unavailable"
        );
        return Ok(false);
    }

    let indices = find_queue_families(instance, surface_loader, surface, device)?;

    if indices.graphics.is_none() || indices.present.is_none() {
        info!(device = device.as_raw(), graphics_family = ?indices.graphics, present_family = ?indices.present, "Rejecting Vulkan device: required queue unavailable");
        return Ok(false);
    }

    // A swapchain needs surface formats and presentation modes.
    let formats = unsafe { surface_loader.get_physical_device_surface_formats(device, surface)? };

    let present_modes =
        unsafe { surface_loader.get_physical_device_surface_present_modes(device, surface)? };

    let suitable = !formats.is_empty() && !present_modes.is_empty();
    info!(
        device = device.as_raw(),
        format_count = formats.len(),
        present_mode_count = present_modes.len(),
        suitable,
        "Vulkan physical device suitability evaluated"
    );
    Ok(suitable)
}

#[instrument(name = "Vulkan - Score Physical Device", skip_all)]
fn score_physical_device(instance: &Instance, device: PhysicalDevice) -> Result<u32> {
    let properties = unsafe { instance.get_physical_device_properties(device) };

    let mut score = match properties.device_type {
        PhysicalDeviceType::DISCRETE_GPU => 20_000,
        PhysicalDeviceType::INTEGRATED_GPU => 10_000,
        PhysicalDeviceType::VIRTUAL_GPU => 5_000,
        PhysicalDeviceType::CPU => 2_500,
        PhysicalDeviceType::OTHER => 500,
        _ => 0,
    };

    let memory = unsafe { instance.get_physical_device_memory_properties(device) };

    let vram = memory
        .memory_heaps
        .iter()
        .take(memory.memory_heap_count as usize)
        .filter(|heap| heap.flags.contains(MemoryHeapFlags::DEVICE_LOCAL))
        .map(|heap| heap.size)
        .sum::<u64>();

    score += ((vram / (1024 * 1024 * 1024)) as u32) * 100;

    let queues = unsafe { instance.get_physical_device_queue_family_properties(device) };

    for queue in &queues {
        let flags = queue.queue_flags;

        // Prefer a dedicated compute queue.
        if flags.contains(ash::vk::QueueFlags::COMPUTE)
            && !flags.contains(ash::vk::QueueFlags::GRAPHICS)
        {
            score += 250;
        }

        // Prefer transfer queues that do not compete with graphics/compute.
        if flags.contains(ash::vk::QueueFlags::TRANSFER) {
            if !flags.contains(ash::vk::QueueFlags::GRAPHICS)
                && !flags.contains(ash::vk::QueueFlags::COMPUTE)
            {
                score += 500;
            } else if !flags.contains(ash::vk::QueueFlags::GRAPHICS) {
                score += 250;
            }
        }
    }

    info!(
        device = device.as_raw(),
        score,
        vram_bytes = vram,
        queue_count = queues.len(),
        "Vulkan physical device scored"
    );
    Ok(score)
}

#[instrument(name = "Vulkan - Pick Physical Device", skip_all)]
pub fn pick_physical_device_heuristic(
    instance: &Instance,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
) -> Result<PhysicalDevice> {
    let devices = unsafe { instance.enumerate_physical_devices()? };
    info!(
        device_count = devices.len(),
        "Enumerated Vulkan physical devices"
    );

    let mut best: Option<(PhysicalDevice, u32)> = None;

    for device in devices {
        if !is_device_suitable(instance, surface_loader, surface, device)? {
            continue;
        }

        let score = score_physical_device(instance, device)?;
        info!(
            device = device.as_raw(),
            score, "Vulkan device candidate accepted"
        );

        if best.map_or(true, |(_, best_score)| score > best_score) {
            best = Some((device, score));
        }
    }

    let selected = best
        .map(|(device, _)| device)
        .ok_or_else(|| anyhow!("No suitable GPU found for Vulkan."))?;
    info!(
        device = selected.as_raw(),
        "Vulkan physical device selected by heuristic"
    );
    Ok(selected)
}
