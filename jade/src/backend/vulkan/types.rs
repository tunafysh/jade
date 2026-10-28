use ash::vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, CullModeFlags, DynamicState, Extent2D, Format,
    FrontFace, Pipeline, PipelineLayout, PolygonMode, PresentModeKHR, PrimitiveTopology,
    SampleCountFlags, SurfaceTransformFlagsKHR,
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

pub struct PipelineConfig {
    pub topology: PrimitiveTopology,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
    pub polygon_mode: PolygonMode,
    pub blending: bool,
    pub samples: SampleCountFlags,
    pub dynamic_states: Vec<DynamicState>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TRIANGLE_LIST,
            cull_mode: CullModeFlags::BACK,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            polygon_mode: PolygonMode::FILL,
            blending: false,
            samples: SampleCountFlags::TYPE_1,
            dynamic_states: vec![DynamicState::VIEWPORT, DynamicState::SCISSOR],
        }
    }
}

// wtf vulkan...
pub struct GraphicsPipeline {
    pub handle: Pipeline,
    pub layout: PipelineLayout,
}
