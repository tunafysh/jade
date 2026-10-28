use anyhow::{Context as _, Result};
use ash::{
    Device,
    vk::{
        ColorComponentFlags, CullModeFlags, DynamicState, Format, FrontFace,
        GraphicsPipelineCreateInfo, Handle, PipelineCache, PipelineColorBlendAttachmentState,
        PipelineColorBlendStateCreateInfo, PipelineDynamicStateCreateInfo,
        PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
        PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
        PipelineRenderingCreateInfo, PipelineShaderStageCreateInfo,
        PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
        PrimitiveTopology, RenderPass, SampleCountFlags, ShaderModule, ShaderStageFlags,
    },
};

use crate::backend::vulkan::types::{GraphicsPipeline, PipelineConfig, SwapchainConfig};
use tracing::{info, instrument};

#[instrument(name = "Vulkan - Create Pipeline Layout", skip_all)]
pub fn create_pipeline_layout(logical_device: &Device) -> Result<PipelineLayout> {
    let pipeline_layout_create_info = PipelineLayoutCreateInfo::default()
        .set_layouts(&[])
        .push_constant_ranges(&[]);

    let pipeline_layout =
        unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }?;
    info!(
        pipeline_layout = pipeline_layout.as_raw(),
        "Vulkan pipeline layout created"
    );
    Ok(pipeline_layout)
}

fn create_rasterization_state(
    cull_mode: CullModeFlags,
    polygon_mode: PolygonMode,
    front_face: FrontFace,
) -> PipelineRasterizationStateCreateInfo<'static> {
    PipelineRasterizationStateCreateInfo::default()
        .cull_mode(cull_mode)
        .polygon_mode(polygon_mode)
        .front_face(front_face)
        .line_width(1.0)
        .rasterizer_discard_enable(false)
        .depth_clamp_enable(false)
        .depth_bias_enable(false)
}

fn create_viewport_state() -> PipelineViewportStateCreateInfo<'static> {
    PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1)
}

fn create_multisampler_state(
    samples: SampleCountFlags,
) -> PipelineMultisampleStateCreateInfo<'static> {
    PipelineMultisampleStateCreateInfo::default().rasterization_samples(samples)
}

fn create_dynamic_state<'a>(
    dynamic_states: &'a [DynamicState],
) -> PipelineDynamicStateCreateInfo<'a> {
    PipelineDynamicStateCreateInfo::default().dynamic_states(dynamic_states)
}

fn create_color_blend_attachment(blend: bool) -> PipelineColorBlendAttachmentState {
    PipelineColorBlendAttachmentState::default()
        .blend_enable(blend)
        .color_write_mask(ColorComponentFlags::RGBA)
}

fn create_color_blend_state<'a>(
    attachments: &'a [PipelineColorBlendAttachmentState],
) -> PipelineColorBlendStateCreateInfo<'a> {
    PipelineColorBlendStateCreateInfo::default()
        .attachments(attachments)
        .logic_op_enable(false)
}

fn create_input_assembly_state(
    topology: PrimitiveTopology,
) -> PipelineInputAssemblyStateCreateInfo<'static> {
    PipelineInputAssemblyStateCreateInfo::default()
        .topology(topology)
        .primitive_restart_enable(false)
}

fn create_vertex_input_state() -> PipelineVertexInputStateCreateInfo<'static> {
    PipelineVertexInputStateCreateInfo::default() //? Default for now.
}

fn create_rendering_info<'a>(color_formats: &'a [Format]) -> PipelineRenderingCreateInfo<'a> {
    PipelineRenderingCreateInfo::default().color_attachment_formats(color_formats)
}

#[instrument(name = "Vulkan - Create Graphics Pipeline", skip_all)]
pub fn create_graphics_pipeline(
    logical_device: &Device,
    shader_module: ShaderModule,
    pipeline_config: PipelineConfig,
    swapchain_config: SwapchainConfig,
) -> Result<GraphicsPipeline> {
    info!(
        format = swapchain_config.image_format.as_raw(),
        samples = pipeline_config.samples.as_raw(),
        topology = pipeline_config.topology.as_raw(),
        dynamic_state_count = pipeline_config.dynamic_states.len(),
        "Creating Vulkan graphics pipeline"
    );
    let vertex_stage = PipelineShaderStageCreateInfo::default()
        .stage(ShaderStageFlags::VERTEX)
        .module(shader_module)
        .name(c"vertex");

    let fragment_stage = PipelineShaderStageCreateInfo::default()
        .stage(ShaderStageFlags::FRAGMENT)
        .module(shader_module)
        .name(c"fragment");

    let stages = [vertex_stage, fragment_stage];

    let layout = create_pipeline_layout(&logical_device)?;
    let attachment = create_color_blend_attachment(pipeline_config.blending);
    let attachments = [attachment];
    let color_blend = create_color_blend_state(&attachments);
    let multisample_state = create_multisampler_state(pipeline_config.samples);
    let dynamic_state = create_dynamic_state(&pipeline_config.dynamic_states);
    let viewport = create_viewport_state();
    let vertex_input_state = create_vertex_input_state();
    let input_assembly_state = create_input_assembly_state(pipeline_config.topology);
    let image_format = [swapchain_config.image_format];
    let mut rendering_info = create_rendering_info(&image_format);
    let rasterizer = create_rasterization_state(
        pipeline_config.cull_mode,
        pipeline_config.polygon_mode,
        pipeline_config.front_face,
    );

    let pipeline_info = GraphicsPipelineCreateInfo::default()
        .layout(layout)
        .color_blend_state(&color_blend)
        .multisample_state(&multisample_state)
        .dynamic_state(&dynamic_state)
        .viewport_state(&viewport)
        .rasterization_state(&rasterizer)
        .stages(&stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .render_pass(RenderPass::null())
        .push_next(&mut rendering_info);

    let pipeline = unsafe {
        logical_device.create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
    };

    let pipeline = match pipeline {
        Ok(pipelines) => pipelines[0],
        Err((pipelines, error)) => {
            unsafe {
                for pipeline in pipelines {
                    logical_device.destroy_pipeline(pipeline, None);
                }
            }

            return Err(error).context("Failed to create graphics pipeline");
        }
    };

    info!(
        pipeline = pipeline.as_raw(),
        pipeline_layout = layout.as_raw(),
        "Vulkan graphics pipeline created"
    );

    Ok(GraphicsPipeline {
        handle: pipeline,
        layout,
    })
}
