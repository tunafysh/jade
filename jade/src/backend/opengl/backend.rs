// dont ask i already went through hell by making the vulkan backend...
use crate::backend::{backend::Backend, window::Window};
use anyhow::Result;
use tokio::runtime::{Builder, Runtime};
use wgpu::{
    Adapter, Backends, Color, CurrentSurfaceTexture, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RequestAdapterOptions, StoreOp, Surface, SurfaceColorSpace, SurfaceConfiguration,
    SurfaceTargetUnsafe, SurfaceTexture, TextureUsages,
};

pub struct OpenGLBackend<'a> {
    instance: Instance,
    surface: Surface<'a>,
    window: Window,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    current_frame: Option<SurfaceTexture>,
}

impl<'a> Backend for OpenGLBackend<'a> {
    fn init(window: Window) -> Result<Self> {
        let runtime = Builder::new_current_thread().build()?;
        let mut window = window;
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::GL,
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

        let configuration = SurfaceConfiguration {
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

        Ok(Self {
            current_frame: None,
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
        })
    }

    fn add_shader(
        &mut self,
        shader: crate::shading::shader::Shader,
        reflection: crate::shading::reflection::ShaderReflection,
    ) {
        todo!()
    }

    fn begin_frame(&mut self) -> Result<()> {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Suboptimal(frame) => frame,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                // Reconfigure the surface and retry next frame.
                return Ok(());
            }
            CurrentSurfaceTexture::Timeout => return Ok(()),
            CurrentSurfaceTexture::Occluded => return Ok(()),
            _ => return Ok(()),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let color_attachment = Some(RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 0.1,
                    g: 0.0,
                    b: 0.2,
                    a: 1.0,
                }),
                store: StoreOp::Store,
            },
        });

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        todo!()
    }

    fn end_frame(&mut self) -> Result<()> {
        todo!()
    }

    fn window(&mut self) -> &mut crate::backend::window::Window {
        todo!()
    }
}
