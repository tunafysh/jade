use anyhow::Result;
use jade::{
    backend::{
        backend::Backend,
        vulkan::backend::VulkanBackend,
        window::{Window, WindowConfig},
    },
    shading::compiler::{Source, compile},
    util::types::BackendType,
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    },
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

struct JadeTestApp {
    backend: Option<VulkanBackend>,
    failure: Option<anyhow::Error>,
}

impl JadeTestApp {
    fn new() -> Self {
        Self {
            backend: None,
            failure: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        error!("{err:#}");
        self.failure = Some(err);
        event_loop.exit();
    }
}

impl ApplicationHandler for JadeTestApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.backend.is_some() || self.failure.is_some() {
            return;
        }

        let result = (|| -> Result<VulkanBackend> {
            let config = WindowConfig::new(1280, 720).title("JADE Vulkan Test");

            let window = Window::create(event_loop, &config)?;
            let backend = VulkanBackend::init(window)?;

            info!("Vulkan backend initialized");
            Ok(backend)
        })();

        match result {
            Ok(mut backend) => {
                backend.window().show();
                backend.window().request_redraw();
                info!(window_id = ?backend.window().id(), "Vulkan test window ready");
                self.backend = Some(backend);
            }
            Err(err) => {
                self.fail(event_loop, err);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };

        if backend.window().id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                info!(window_id = ?window_id, "Window redraw requested");
                    let result = backend
                        .begin_frame()
                        .and_then(|()| backend.draw())
                        .and_then(|()| backend.end_frame());

                    if let Err(err) = result {
                        self.fail(event_loop, err);
                    }
            }

            WindowEvent::Resized(_) => {}

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.backend.is_none() && self.failure.is_some() {
            event_loop.exit();
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_level(true)
        .init();

    let shader = include_str!("../../assets/default.wgsl");
    let source = Source::Source(shader.to_string());

    for backend_type in [
        BackendType::OpenGL,
        BackendType::Vulkan,
        BackendType::WebGPU,
    ] {
        info!("Compiling sample shader for {backend_type:?}");

        let (compiled_shader, reflection) = compile(source.clone(), backend_type)?;

        info!(
            "{backend_type:?} shader: {compiled_shader:#?}\n\
             Reflection: {reflection:#?}"
        );
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = JadeTestApp::new();
    event_loop.run_app(&mut app)?;

    if let Some(err) = app.failure {
        return Err(err);
    }

    info!("JADE Vulkan test harness exiting");
    Ok(())
}
