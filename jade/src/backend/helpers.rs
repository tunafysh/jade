use wgpu::*;

use crate::graphics::vertex::CVertex; // dont ask i got tired ok?

pub fn create_primitive_state() -> PrimitiveState {
    PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false, // liberal.
    }
}

pub fn create_render_pipeline_descriptor<'a>(
    vertex: VertexState<'a>,
    fragment: FragmentState<'a>,
    layout: &'a PipelineLayout,
    primitive: PrimitiveState,
) -> RenderPipelineDescriptor<'a> {
    RenderPipelineDescriptor {
        label: None,
        layout: Some(layout),
        vertex,
        primitive,
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(fragment),
        multiview_mask: None,
        cache: None,
    }
}

pub fn create_compute_pipeline_descriptor<'a>(
    module: &'a ShaderModule,
    entry_point: &'a str,
    layout: &'a PipelineLayout,
) -> ComputePipelineDescriptor<'a> {
    ComputePipelineDescriptor {
        label: None,
        layout: Some(layout),
        module: module,
        entry_point: Some(entry_point),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    }
}

pub fn create_vertex_buffer_layout<'a>() -> VertexBufferLayout<'a> {
    VertexBufferLayout {
        array_stride: size_of::<CVertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            // Position
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            },
            // Normal
            VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
            // UV
            VertexAttribute {
                offset: 24,
                shader_location: 2,
                format: VertexFormat::Float32x2,
            },
            // Color
            VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: VertexFormat::Float32x3,
            },
        ],
    }
}
