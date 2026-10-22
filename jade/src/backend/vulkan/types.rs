use ash::vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, PresentModeKHR,
    SurfaceTransformFlagsKHR,
};

#[derive(Copy, Clone)]
pub struct SwapchainConfig {
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub present_mode: PresentModeKHR,
    pub image_count: u32,
    pub composite_alpha: CompositeAlphaFlagsKHR,
    pub transform_flags: SurfaceTransformFlagsKHR,
}

pub struct QueueFamilyIndices {
    pub graphics: Option<u32>,
    pub present: Option<u32>,
    pub compute: Option<u32>,
    pub transfer: Option<u32>,
}
