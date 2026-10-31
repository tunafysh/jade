use std::collections::HashMap;

// dont ask i already went through hell by making the vulkan backend...
use crate::{
    backend::{
        draw_command::DrawCommand,
        helpers::{
            create_compute_pipeline_descriptor, create_primitive_state,
            create_render_pipeline_descriptor, create_vertex_buffer_layout,
        },
        types::ShaderResource,
        window::Window,
    },
    graphics::vertex::CVertex,
    shading::shader::ShaderStage,
};
use anyhow::{Result, anyhow};
use bytemuck::cast_slice;
use tokio::runtime::Builder;
use wgpu::*;

struct Frame {
    texture: SurfaceTexture,
    view: TextureView,
    encoder: CommandEncoder,
}

#[allow(dead_code)] // will need the instance and adapter later maybe
pub struct Backend<'a> {
    instance: Instance,
    surface: Surface<'a>,
    window: Window,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    current_frame: Option<Frame>,
    shader_modules: HashMap<String, ShaderResource>,
    render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    compute_pipelines: HashMap<String, wgpu::ComputePipeline>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl<'a> Backend<'a> {
    pub fn init(window: Window) -> Result<Self> {
        let runtime = Builder::new_current_thread().build()?;
        let window = window;
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..InstanceDescriptor::new_without_display_handle()
        });

        let handle = window.handle();
        let target = unsafe { SurfaceTargetUnsafe::from_display_and_window(handle, handle) }?;
        let surface = unsafe { instance.create_surface_unsafe(target) }?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            ..Default::default()
        });

        let adapter = runtime.block_on(adapter)?;

        let info = adapter.get_info();

        tracing::info!(
            name = %info.name,
            vendor = info.vendor,
            device = info.device,
            backend = ?info.backend,
            device_type = ?info.device_type,
            "Selected GPU adapter"
        );

        let device_descriptor = DeviceDescriptor {
            label: Some("JADE Device"),
            ..Default::default()
        };
        let (device, queue) = runtime.block_on(adapter.request_device(&device_descriptor))?;

        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: SurfaceColorSpace::Srgb,
            width: window.width(),
            height: window.height(),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("JADE Vertex Buffer"),
            size: 1024 * 1024, // temporary 1MiB allocation
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(
            &BufferDescriptor {
                label: Some("JADE Index Buffer"),
                size: 1024 * 1024, // same as 8 lines before
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );

        Ok(Self {
            current_frame: None,
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            shader_modules: HashMap::new(),
            render_pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            vertex_buffer,
            index_buffer
        })
    }

    pub fn add_shader(
        &mut self,
        shader: crate::shading::shader::Shader,
        reflection: crate::shading::reflection::ShaderReflection,
    ) {
        let source = shader.output.source;
        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&shader.name),
            source: ShaderSource::Wgsl(source.into()),
        });

        self.shader_modules
            .insert(shader.name, ShaderResource { module, reflection });
    }

    pub fn add_graphics_pipeline(&mut self, shader_name: String) -> Result<()> {
        let shader = self
            .shader_modules
            .get(&shader_name)
            .ok_or(anyhow!("Such shader does not exist. Please add it first."))?;
        let mut entry_points = shader.reflection.entry_points.iter();
        let vertex_name = entry_points
            .find(|e| e.stage == ShaderStage::Vertex)
            .ok_or(anyhow!("Shader does not contain a necessary vertex entry."))?
            .name
            .clone();
        let fragment_name = entry_points
            .find(|e| e.stage == ShaderStage::Fragment)
            .ok_or(anyhow!("Shader does not contain a necessary vertex entry."))?
            .name
            .clone();
        let target = ColorTargetState {
            format: self.config.format,
            blend: Some(BlendState::ADDITIVE),
            write_mask: ColorWrites::ALL,
        };

        let vertex_state = VertexState {
            module: &shader.module,
            entry_point: Some(&vertex_name),
            compilation_options: Default::default(),
            buffers: &[Some(create_vertex_buffer_layout())],
        };

        let fragment_state = FragmentState {
            module: &shader.module,
            entry_point: Some(&fragment_name),
            compilation_options: Default::default(),
            targets: &[Some(target)],
        };

        let primitive_state = create_primitive_state();

        let layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("JADE Pipeline Layout"),
                bind_group_layouts: &[None],
                immediate_size: 0,
            });

        let descriptor = create_render_pipeline_descriptor(
            vertex_state,
            fragment_state,
            &layout,
            primitive_state,
        );

        let pipeline = self.device.create_render_pipeline(&descriptor);

        self.render_pipelines.insert(shader_name, pipeline);

        Ok(())
    }

    pub fn add_compute_pipeline(&mut self, shader_name: String) -> Result<()> {
        let shader = self
            .shader_modules
            .get(&shader_name)
            .ok_or(anyhow!("Shader not found"))?;
        let mut entry_points = shader.reflection.entry_points.iter();
        let compute_fn = entry_points
            .find(|e| e.stage == ShaderStage::Compute)
            .ok_or(anyhow!("Compute entry point not found."))?
            .name
            .clone();

        let layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("JADE Pipeline Layout"),
                bind_group_layouts: &[None],
                immediate_size: 0,
            });
        let descriptor = create_compute_pipeline_descriptor(&shader.module, &compute_fn, &layout);

        let pipeline = self.device.create_compute_pipeline(&descriptor);
        self.compute_pipelines.insert(shader_name, pipeline);

        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<bool> {
        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Suboptimal(frame) => frame,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return Ok(false),
            _ => return Ok(false),
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("JADE Command Encoder"),
            });

        let frame = Frame {
            texture: surface_texture,
            view,
            encoder,
        };

        self.current_frame = Some(frame);

        Ok(true)
    }

    pub fn draw(&mut self, command: DrawCommand) -> Result<()> {
        let vertices: Vec<CVertex> = command.mesh.vertices.iter().map(CVertex::from).collect();

        let pipeline = self
            .render_pipelines
            .get(&command.shader)
            .ok_or_else(|| anyhow!("Such render pipeline does not exist. Please add it first."))?;

        let frame = self
            .current_frame
            .as_mut()
            .ok_or_else(|| anyhow!("Cannot draw without an active frame."))?;

        let color_attachment = Some(RenderPassColorAttachment {
            view: &frame.view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 0.03,
                    g: 0.03,
                    b: 0.04,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
        });

        let mut render_pass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("JADE Render Pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        self.queue
            .write_buffer(&self.vertex_buffer, 0, cast_slice(&vertices));

        self.queue.write_buffer(&self.index_buffer, 0, cast_slice(&command.mesh.indices));

        render_pass.set_pipeline(pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..command.mesh.indices.len() as u32, 0 ,0..1);

        drop(render_pass);
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<()> {
        let frame = self
            .current_frame
            .take()
            .ok_or_else(|| anyhow!("Cannot end a frame that has not begun."))?;
        self.queue.submit(Some(frame.encoder.finish()));
        self.queue.present(frame.texture);
        Ok(())
    }

    pub fn window(&mut self) -> &mut crate::backend::window::Window {
        &mut self.window
    }
}
