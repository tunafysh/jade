use std::ffi::{CStr, CString};

use anyhow::{Result, anyhow, bail};
use ash::{
    Device, Entry, Instance,
    khr::{surface::Instance as SurfaceInstance, swapchain::Device as SwapchainLoader},
    vk::{
        self, ColorSpaceKHR, CompositeAlphaFlagsKHR, DeviceCreateInfo, DeviceQueueCreateInfo,
        Format, Handle, ImageUsageFlags, MemoryHeapFlags, PhysicalDevice, PhysicalDeviceType,
        PresentModeKHR, QueueFlags, SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
use glfw::ffi::VkResult_VK_SUCCESS;
use tracing::{debug, info, info_span, instrument};

use crate::backend::{
    vulkan::types::{QueueFamilyIndices, SwapchainConfig},
    window::Window,
};

const REQUIRED_INSTANCE_EXTENSIONS: [&str; 1] = ["VK_EXT_swapchain_colorspace"];
const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

#[instrument(name = "Vulkan - Create Instance", skip_all)]
pub fn create_instance(window: &mut Window) -> Result<(Entry, Instance)> {
    let entry = unsafe { Entry::load() }?;

    let mut extensions = window
        .instance()
        .get_required_instance_extensions()
        .ok_or_else(|| anyhow::anyhow!("GLFW does not support Vulkan"))?;

    for extension in REQUIRED_INSTANCE_EXTENSIONS {
        extensions.push(extension.to_string());
    }

    let extension_names = extensions
        .iter()
        .map(|extension| CString::new(extension.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    let extension_name_pointers = extension_names
        .iter()
        .map(|extension| extension.as_ptr())
        .collect::<Vec<_>>();

    let validation_layers = [VALIDATION_LAYER.as_ptr()];

    let app_name = CString::new("JADE")?;
    let engine_name = CString::new("JADE")?;

    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 3, 0),
        p_application_name: app_name.as_ptr(),
        p_engine_name: engine_name.as_ptr(),
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        pp_enabled_extension_names: extension_name_pointers.as_ptr(),
        enabled_extension_count: extension_name_pointers.len() as u32,
        pp_enabled_layer_names: validation_layers.as_ptr(),
        enabled_layer_count: validation_layers.len() as u32,
        ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&create_info, None) }?;

    Ok((entry, instance))
}

#[instrument(name = "Vulkan - Create Surface", skip_all)]
pub fn create_surface(window: &mut Window, instance: &mut Instance) -> Result<SurfaceKHR> {
    let mut surface = std::ptr::null_mut();
    unsafe {
        let raw_instance = instance.handle().as_raw() as *mut _;
        let result =
            window
                .handle()
                .create_window_surface(raw_instance, std::ptr::null(), &mut surface);
        if result != VkResult_VK_SUCCESS {
            bail!("Failed to create window surface.")
        }
    }

    Ok(SurfaceKHR::from_raw(surface as u64))
}

pub fn find_queue_families(
    instance: &Instance,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
    device: PhysicalDevice,
) -> Result<QueueFamilyIndices> {
    let families = unsafe { instance.get_physical_device_queue_family_properties(device) };

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

        let present_support =
            unsafe { surface_loader.get_physical_device_surface_support(device, index, surface)? };

        if present_support {
            indices.present = Some(index);
        }

        if family.queue_flags.contains(QueueFlags::COMPUTE) {
            indices.compute = Some(index);
        }

        if family.queue_flags.contains(QueueFlags::TRANSFER) {
            indices.transfer = Some(index);
        }
    }

    Ok(indices)
}

pub fn is_device_suitable(
    instance: &Instance,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
    device: PhysicalDevice,
) -> Result<bool> {
    let indices = find_queue_families(instance, surface_loader, surface, device)?;

    Ok(indices.graphics.is_some() && indices.present.is_some())
}

pub fn score_physical_device(
    instance: &Instance,
    device: PhysicalDevice,
    surface_loader: &SurfaceInstance,
    surface: SurfaceKHR,
) -> Result<u32> {
    let mut score = 0;

    //Score based on what type of gpu it is
    let properties = unsafe { instance.get_physical_device_properties(device) };

    score += match properties.device_type {
        PhysicalDeviceType::DISCRETE_GPU => 20000,
        PhysicalDeviceType::INTEGRATED_GPU => 10000,
        PhysicalDeviceType::VIRTUAL_GPU => 5000,
        PhysicalDeviceType::CPU => 2500,
        PhysicalDeviceType::OTHER => 500,
        _ => 0,
    };

    // score based on how much vram it has
    let memory = unsafe { instance.get_physical_device_memory_properties(device) };

    let mut vram = 0;

    for heap in memory
        .memory_heaps
        .iter()
        .take(memory.memory_heap_count as usize)
    {
        if heap.flags.contains(MemoryHeapFlags::DEVICE_LOCAL) {
            vram += heap.size;
        }
    }

    let vram_gib = vram / (1024 * 1024 * 1024);

    score += (vram_gib as u32) * 100;

    // Add score for additional queues like transfer and compute. (extra if dedicated)
    let families = unsafe { instance.get_physical_device_queue_family_properties(device) };

    for (index, queue) in families.iter().enumerate() {
        let index = index as u32;

        let has_present =
            unsafe { surface_loader.get_physical_device_surface_support(device, index, surface)? };

        if queue.queue_flags.contains(QueueFlags::COMPUTE) {
            if !has_present
                || queue.queue_flags.contains(QueueFlags::TRANSFER)
                    && queue.queue_flags.contains(QueueFlags::GRAPHICS)
            {
                score += 250;
            };
            score += 250;
        }

        if queue.queue_flags.contains(QueueFlags::TRANSFER) {
            if !has_present
                || queue.queue_flags.contains(QueueFlags::COMPUTE)
                    && queue.queue_flags.contains(QueueFlags::GRAPHICS)
            {
                score += 250;
            };
            score += 250;
        };
    }

    Ok(score)
}

#[instrument(name = "Vulkan - Pick Physical Device", skip_all)]
pub fn pick_physical_device_heuristic(
    instance: &mut Instance,
    surface_loader: &SurfaceInstance,
    surface: &mut SurfaceKHR,
) -> Result<PhysicalDevice> {
    let devices = unsafe { instance.enumerate_physical_devices()? };

    let physical_device = devices
        .iter()
        .copied()
        .filter(|device| {
            is_device_suitable(instance, &surface_loader, *surface, *device).unwrap_or(false)
        })
        .max_by_key(|device| {
            score_physical_device(instance, *device, &surface_loader, *surface).unwrap_or(0)
        })
        .ok_or_else(|| anyhow!("No suitable GPU found for vulkan."))?;

    Ok(physical_device)
}

#[instrument(name = "Vulkan - Create Logical Device", skip_all)]
pub fn create_logical_device(
    instance: &mut Instance,
    physical_device: PhysicalDevice,
    surface_loader: &SurfaceInstance,
    surface: &mut SurfaceKHR,
) -> Result<(Device, QueueFamilyIndices)> {
    let extensions = [ash::khr::swapchain::NAME.as_ptr()];

    let queue_priority = [1.0];

    let queue_families = find_queue_families(instance, surface_loader, *surface, physical_device)?;

    let graphics_family = queue_families
        .graphics
        .ok_or_else(|| anyhow::anyhow!("No graphics queue family"))?;

    let queue_create_info: [DeviceQueueCreateInfo; 1] = [DeviceQueueCreateInfo::default()
        .queue_family_index(graphics_family)
        .queue_priorities(&queue_priority)];

    let device_create_info = DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_info)
        .enabled_extension_names(&extensions);

    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    Ok((logical_device, queue_families))
}

#[instrument(name = "Vulkan - Check HDR", skip_all)]
pub fn check_hdr(
    physical_device: PhysicalDevice,
    surface_loader: &SurfaceInstance,
    surface: &mut SurfaceKHR,
) -> Result<bool> {
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, *surface) }?;

    let supports = select_surface_format(&formats, true).is_ok();

    info!(supports, "HDR surface format support checked");

    Ok(supports)
}

pub fn select_surface_format(
    formats: &[vk::SurfaceFormatKHR],
    enable_hdr: bool,
) -> Result<(Format, ColorSpaceKHR)> {
    let span = info_span!("Vulkan - Select Surface Format", hdr = enable_hdr);
    let _guard = span.enter();

    if formats.len() == 1 && formats[0].format == Format::UNDEFINED {
        let color_space = formats[0].color_space;
        let is_hdr_color_space = matches!(
            color_space,
            ColorSpaceKHR::HDR10_ST2084_EXT | ColorSpaceKHR::HDR10_HLG_EXT
        );

        if enable_hdr == is_hdr_color_space {
            let format = if enable_hdr {
                Format::A2B10G10R10_UNORM_PACK32
            } else {
                Format::B8G8R8A8_SRGB
            };
            return Ok((format, color_space));
        }

        return Err(anyhow!(
            "Surface does not advertise a compatible {} color space",
            if enable_hdr { "HDR" } else { "SDR" }
        ));
    }

    let preferred_pairs = if enable_hdr {
        [
            (
                Format::A2B10G10R10_UNORM_PACK32,
                ColorSpaceKHR::HDR10_ST2084_EXT,
            ),
            (
                Format::A2R10G10B10_UNORM_PACK32,
                ColorSpaceKHR::HDR10_ST2084_EXT,
            ),
            (
                Format::A2B10G10R10_UNORM_PACK32,
                ColorSpaceKHR::HDR10_HLG_EXT,
            ),
            (
                Format::A2R10G10B10_UNORM_PACK32,
                ColorSpaceKHR::HDR10_HLG_EXT,
            ),
        ]
    } else {
        [
            (Format::B8G8R8A8_SRGB, ColorSpaceKHR::SRGB_NONLINEAR),
            (Format::R8G8B8A8_SRGB, ColorSpaceKHR::SRGB_NONLINEAR),
            (Format::B8G8R8A8_UNORM, ColorSpaceKHR::SRGB_NONLINEAR),
            (Format::R8G8B8A8_UNORM, ColorSpaceKHR::SRGB_NONLINEAR),
        ]
    };

    preferred_pairs
        .into_iter()
        .find(|(format, color_space)| {
            formats.iter().any(|available| {
                available.format == *format && available.color_space == *color_space
            })
        })
        .ok_or_else(|| {
            anyhow!(
                "No supported {} surface format/color-space pair",
                if enable_hdr { "HDR" } else { "SDR" }
            )
        })
}

pub fn select_composite_alpha(supported: CompositeAlphaFlagsKHR) -> Result<CompositeAlphaFlagsKHR> {
    let span = info_span!(
        "Vulkan - Select Composite Alpha",
        supported = supported.as_raw()
    );
    let _guard = span.enter();

    [
        CompositeAlphaFlagsKHR::OPAQUE,
        CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
        CompositeAlphaFlagsKHR::POST_MULTIPLIED,
        CompositeAlphaFlagsKHR::INHERIT,
    ]
    .into_iter()
    .find(|candidate| supported.contains(*candidate))
    .ok_or_else(|| anyhow!("Surface reports no supported composite alpha mode"))
}

#[instrument(name = "Vulkan - Create Swapchain Config", skip_all)]
pub fn create_swapchain_config(
    physical_device: PhysicalDevice,
    surface_loader: &SurfaceInstance,
    surface: &mut SurfaceKHR,
    enable_hdr: bool,
    vsync: bool,
) -> Result<SwapchainConfig> {
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, *surface)
    }?;

    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, *surface)
    }?;

    let present_mode = match vsync {
        true if present_modes.contains(&PresentModeKHR::MAILBOX) => PresentModeKHR::MAILBOX,
        false if present_modes.contains(&PresentModeKHR::IMMEDIATE) => PresentModeKHR::IMMEDIATE,
        false if present_modes.contains(&PresentModeKHR::MAILBOX) => PresentModeKHR::MAILBOX,
        _ => PresentModeKHR::FIFO,
    };

    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, *surface) }?;
    let (image_format, color_space) = select_surface_format(&formats, enable_hdr)?;
    let composite_alpha = select_composite_alpha(capabilities.supported_composite_alpha)?;

    debug!(format_count = formats.len(), "Surface formats advertised");
    info!(
        image_format = image_format.as_raw(),
        color_space = color_space.as_raw(),
        composite_alpha = composite_alpha.as_raw(),
        present_mode = present_mode.as_raw(),
        image_count = capabilities.min_image_count,
        width = capabilities.current_extent.width,
        height = capabilities.current_extent.height,
        "Vulkan swapchain configuration selected"
    );

    let swapchain_config = SwapchainConfig {
        image_extent: capabilities.current_extent,
        image_count: capabilities.min_image_count,
        composite_alpha,
        transform_flags: capabilities.current_transform,
        image_color_space: color_space,
        image_format,
        present_mode,
    };

    Ok(swapchain_config)
}

/// Creates the Vulkan swapchain for a window surface.
///
/// # `SwapchainCreateInfoKHR` fields
///
/// | Field | Purpose |
/// |---|---|
/// | `surface` | Window surface to present into. |
/// | `min_image_count` | Number of swapchain images to request. |
/// | `image_format` | Pixel format of the swapchain images. |
/// | `image_color_space` | Color space used for presentation, such as SDR sRGB or HDR10 ST2084. |
/// | `image_extent` | Width and height of swapchain images in pixels. |
/// | `image_array_layers` | Number of image layers. Usually `1` for a normal 2D swapchain. |
/// | `image_usage` | How swapchain images will be used, usually `COLOR_ATTACHMENT`. |
/// | `image_sharing_mode` | How graphics and presentation queues share images: `EXCLUSIVE` or `CONCURRENT`. |
/// | `queue_family_indices` | Queue family indices used with `CONCURRENT` sharing. |
/// | `pre_transform` | Surface transform to apply, usually `current_transform`. |
/// | `composite_alpha` | Alpha compositing mode. Choose one supported by the surface, often `OPAQUE`. |
/// | `present_mode` | Presentation behavior, such as `FIFO`, `MAILBOX`, or `IMMEDIATE`. |
/// | `clipped` | Allows pixel data obscured by other windows to be discarded. |
/// | `old_swapchain` | Previous swapchain being replaced during recreation, or a null handle initially. |
#[instrument(name = "Vulkan - Create Swapchain", skip_all)]
pub fn init_swapchain(
    instance: &mut Instance,
    device: &Device,
    surface: &mut SurfaceKHR,
    config: SwapchainConfig,
) -> Result<(SwapchainLoader, SwapchainKHR)> {
    let create_info = SwapchainCreateInfoKHR::default()
        .surface(*surface)
        .min_image_count(config.image_count)
        .image_format(config.image_format)
        .image_color_space(config.image_color_space)
        .image_extent(config.image_extent)
        .image_array_layers(1)
        .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(SharingMode::EXCLUSIVE)
        .pre_transform(config.transform_flags)
        .composite_alpha(config.composite_alpha)
        .present_mode(config.present_mode)
        .clipped(true);

    info!(
        image_format = config.image_format.as_raw(),
        color_space = config.image_color_space.as_raw(),
        composite_alpha = config.composite_alpha.as_raw(),
        present_mode = config.present_mode.as_raw(),
        "Creating Vulkan swapchain"
    );

    let swapchain_loader = SwapchainLoader::new(instance, device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;
    Ok((swapchain_loader, swapchain))
}
