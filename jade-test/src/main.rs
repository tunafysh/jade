use anyhow::Result;
use jade::{
    backend::{
        backend::Backend, draw_command::DrawCommand, window::{Window, WindowConfig},
    }, graphics::mesh::Mesh, shading::compiler::{ShaderSource, Snippet, compile}, winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    },
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const TRIANGLE_SHADER: &str = include_str!("../../assets/triangle.wgsl");

struct JadeTestApp<'a> {
    window: Option<Window>,
    backend: Option<Backend<'a>>,
    failure: Option<anyhow::Error>,
}

impl<'a> JadeTestApp<'a> {
    fn new() -> Self {
        Self {
            window: None,
            backend: None,
            failure: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        error!("{err:#}");
        self.failure = Some(err);
        event_loop.exit();
    }

    fn initialize_backend(&mut self, _event_loop: &ActiveEventLoop) -> Result<()> {
        let window = self
            .window
            .take()
            .ok_or_else(|| anyhow::anyhow!("No window available"))?;

        info!("Initializing WGPU backend");

        let mut backend = Backend::init(window)?;

        let (shader, reflection) = compile(ShaderSource::Snippet(Snippet {
            name: "triangle".to_string(),
            source: TRIANGLE_SHADER.to_string(),
        }))?;

        backend.add_shader(shader, reflection);
        backend.add_graphics_pipeline("triangle".to_string())?;

        info!("WGPU backend initialized");

        backend.window().request_redraw();
        self.backend = Some(backend);

        Ok(())
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };

        let result = (|| -> Result<()> {
            if !backend.begin_frame()? {
                backend.window().request_redraw();
                return Ok(());
            }

            

            let command = DrawCommand {
                shader: "triangle".to_string(),
                ..Default::default()
            };

            backend.draw(command)?;
            backend.end_frame()?;

            backend.window().request_redraw();

            Ok(())
        })();

        if let Err(err) = result {
            self.fail(event_loop, err);
        }
    }
}

impl<'a> ApplicationHandler for JadeTestApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() || self.backend.is_some() || self.failure.is_some() {
            return;
        }

        let result = (|| -> Result<()> {
            let config = WindowConfig::new(1280, 720).title("JADE WGPU Test");

            let window = Window::create(event_loop, &config)?;
            window.show();

            info!(window_id = ?window.id(), "Window created");

            self.window = Some(window);

            self.window
                .as_ref()
                .expect("window was just stored")
                .request_redraw();

            Ok(())
        })();

        if let Err(err) = result {
            self.fail(event_loop, err);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let matches_window = self
            .backend
            .as_mut()
            .map(|backend| backend.window().id() == window_id)
            .or_else(|| self.window.as_ref().map(|window| window.id() == window_id))
            .unwrap_or(false);

        if !matches_window {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if self.backend.is_none() {
                    if let Err(err) = self.initialize_backend(event_loop) {
                        self.fail(event_loop, err);
                    }
                } else {
                    self.render(event_loop);
                }
            }

            WindowEvent::Resized(_) => {
                // Handle surface reconfiguration here if required.
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // ControlFlow::Wait lets the event loop sleep until an event arrives.
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_level(true)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = JadeTestApp::new();
    event_loop.run_app(&mut app)?;

    if let Some(err) = app.failure {
        return Err(err);
    }

    info!("JADE test harness exiting");

    Ok(())
}
