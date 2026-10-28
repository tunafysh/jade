use crate::backend::vulkan::types::SwapchainConfig;
use anyhow::{Result, anyhow};
use ash::{
    Device, Instance,
    khr::{surface::Instance as SurfaceInstance, swapchain::Device as SwapchainLoader},
    vk::{
        ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, Handle, ImageUsageFlags,
        ImageView, ImageViewCreateInfo, ImageViewType, PhysicalDevice, PresentModeKHR, SharingMode,
        SurfaceFormatKHR, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
use tracing::{debug, info, info_span, instrument};

#[instrument(name = "Vulkan - Create Swapchain", skip_all)]
pub fn create_swapchain(
    instance: &mut Instance,
    device: &Device,
    surface: &mut SurfaceKHR,
    config: SwapchainConfig,
) -> Result<(SwapchainLoader, SwapchainKHR)> {
    info!(
        width = config.image_extent.width,
        height = config.image_extent.height,
        image_count = config.image_count,
        format = config.image_format.as_raw(),
        present_mode = config.present_mode.as_raw(),
        "Creating Vulkan swapchain"
    );
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
    info!(swapchain = swapchain.as_raw(), "Vulkan swapchain created");
    Ok((swapchain_loader, swapchain))
}

#[instrument(name = "Vulkan - Create Swapchain Image Views", skip_all)]
pub fn create_swapchain_image_views(
    device: &Device,
    swapchain_loader: &SwapchainLoader,
    swapchain: SwapchainKHR,
    format: Format,
) -> Result<Vec<ImageView>> {
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;
    info!(
        swapchain = swapchain.as_raw(),
        image_count = images.len(),
        "Vulkan swapchain images acquired"
    );
    let views: Result<Vec<_>> = images
        .into_iter()
        .map(|image| {
            let create_info = ImageViewCreateInfo::default()
                .view_type(ImageViewType::TYPE_2D)
                .format(format)
                .image(image);
            let view = unsafe { device.create_image_view(&create_info, None) }?;
            info!(
                image = image.as_raw(),
                image_view = view.as_raw(),
                "Vulkan swapchain image view created"
            );
            Ok(view)
        })
        .collect();
    info!(
        swapchain = swapchain.as_raw(),
        image_view_count = views.as_ref().map(Vec::len).unwrap_or(0),
        "Vulkan swapchain image views complete"
    );
    views
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
    formats: &[SurfaceFormatKHR],
    enable_hdr: bool,
) -> Result<(Format, ColorSpaceKHR)> {
    let span = info_span!("Vulkan - Select Surface Format", hdr = enable_hdr);
    let _guard = span.enter();
    if formats.len() == 1 && formats[0].format == Format::UNDEFINED {
        let color_space = formats[0].color_space;
        let is_hdr = matches!(
            color_space,
            ColorSpaceKHR::HDR10_ST2084_EXT | ColorSpaceKHR::HDR10_HLG_EXT
        );
        if enable_hdr == is_hdr {
            return Ok((
                if enable_hdr {
                    Format::A2B10G10R10_UNORM_PACK32
                } else {
                    Format::B8G8R8A8_SRGB
                },
                color_space,
            ));
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
    window_extent: Extent2D,
) -> Result<SwapchainConfig> {
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, *surface)
    }?;
    info!(
        min_image_count = capabilities.min_image_count,
        max_image_count = capabilities.max_image_count,
        current_width = capabilities.current_extent.width,
        current_height = capabilities.current_extent.height,
        requested_width = window_extent.width,
        requested_height = window_extent.height,
        "Vulkan surface capabilities queried"
    );
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, *surface)
    }?;
    let present_mode = match vsync {
        true if present_modes.contains(&PresentModeKHR::MAILBOX) => PresentModeKHR::MAILBOX,
        false if present_modes.contains(&PresentModeKHR::IMMEDIATE) => PresentModeKHR::IMMEDIATE,
        false if present_modes.contains(&PresentModeKHR::MAILBOX) => PresentModeKHR::MAILBOX,
        _ => PresentModeKHR::FIFO,
    };
    info!(
        present_mode = present_mode.as_raw(),
        vsync,
        present_mode_count = present_modes.len(),
        "Vulkan present mode selected"
    );
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, *surface) }?;
    let (image_format, color_space) = select_surface_format(&formats, enable_hdr)?;
    info!(
        format = image_format.as_raw(),
        color_space = color_space.as_raw(),
        hdr = enable_hdr,
        format_count = formats.len(),
        "Vulkan surface format selected"
    );
    let composite_alpha = select_composite_alpha(capabilities.supported_composite_alpha)?;
    let image_extent = if capabilities.current_extent.width == u32::MAX {
        Extent2D {
            width: window_extent.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: window_extent.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    } else {
        capabilities.current_extent
    };
    debug!(format_count = formats.len(), "Surface formats advertised");
    info!(
        image_format = image_format.as_raw(),
        color_space = color_space.as_raw(),
        composite_alpha = composite_alpha.as_raw(),
        present_mode = present_mode.as_raw(),
        image_count = capabilities.min_image_count,
        width = image_extent.width,
        height = image_extent.height,
        "Vulkan swapchain configuration selected"
    );
    Ok(SwapchainConfig {
        image_extent,
        image_count: capabilities.min_image_count,
        composite_alpha,
        transform_flags: capabilities.current_transform,
        image_color_space: color_space,
        image_format,
        present_mode,
    })
}
