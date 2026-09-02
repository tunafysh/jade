use anyhow::Result;
use glfw::Context;
use glow::{COLOR_BUFFER_BIT, Context as GlowContext, DEPTH_BUFFER_BIT, HasContext};
use tracing::{info, info_span};

use crate::backend::{
    backend::Backend,
    draw_command::DrawCommand,
    window::Window,
};

pub struct OpenGLBackend {
    window: Window,
    gl: GlowContext,

    // TODO: implement actual resource managers
    // meshes: HashMap<String, OpenGLMesh>,
    // shaders: HashMap<String, OpenGLShader>,
}

impl OpenGLBackend {
    fn init_context(window: &mut Window) -> Result<GlowContext> {
        info!("Initializing GLFW Context");

        window.handle().make_current();

        if !window.handle().is_current() {
            anyhow::bail!("Failed to make GLFW context current");
        }

        info!("Initializing OpenGL Context");

        let gl = unsafe {
            GlowContext::from_loader_function(|symbol| {
                window
                    .handle()
                    .get_proc_address(symbol)
                    .map_or(std::ptr::null(), |p| p as *const _)
            })
        };

        let version = gl.version();

        info!(
            "OpenGL Version: {}.{}{}{}",
            version.major,
            version.minor,
            version
                .revision
                .map(|revision| format!(" Rev: {revision}"))
                .unwrap_or_default(),
            if version.is_embedded {
                " (Embedded)"
            } else {
                ""
            }
        );

        unsafe {
            info!(
                "OpenGL Vendor: {}",
                gl.get_parameter_string(glow::VENDOR)
            );

            info!(
                "OpenGL Renderer: {}",
                gl.get_parameter_string(glow::RENDERER)
            );

            info!(
                "GLSL Version: {}",
                gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION)
            );
        }

        Ok(gl)
    }

    fn init_state(&self) -> Result<()> {
        unsafe {
            self.gl.enable(glow::DEPTH_TEST);

            self.gl.clear_color(
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }

        self.update_viewport()?;

        Ok(())
    }

    fn update_viewport(&self) -> Result<()> {
        let width = self.window.width() as i32;
        let height = self.window.height() as i32;

        unsafe {
            self.gl.viewport(
                0,
                0,
                width,
                height,
            );
        }

        Ok(())
    }
}

impl Backend for OpenGLBackend {
    fn init(mut window: Window) -> Result<Self> {
        let span = info_span!("OpenGL Initialization");
        let _guard = span.enter();

        let gl = Self::init_context(&mut window)?;

        let backend = Self {
            window,
            gl,
        };

        backend.init_state()?;

        Ok(backend)
    }

    fn add_shader(&mut self) {
        // TODO
    }

    fn begin_frame(&mut self) -> Result<()> {
        self.update_viewport()?;

        unsafe{
            self.gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }
        Ok(())
    }

    fn draw(&mut self, command: &DrawCommand) {
        // TODO
    }

    fn end_frame(&mut self) {
        self.window.swap_buffers();
    }

    fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}