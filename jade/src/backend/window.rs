use crate::util::types::BackendType;
use glfw::{Context, Glfw, PWindow, WindowHint, WindowMode};

pub struct Window {
    width: u32,
    height: u32,
    title: String,

    instance: Glfw,
    handle: PWindow,
}

impl Window {
    pub fn new(
        backend_type: BackendType,
        width: u32,
        height: u32,
        title: Option<&str>,
        mode: Option<WindowMode>,
    ) -> Result<Self, glfw::InitError> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;

        match backend_type {
            BackendType::OpenGL => {
                glfw.window_hint(WindowHint::ContextVersion(4, 6));
                glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
            }

            BackendType::Vulkan => {
                glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
            }
        }

        let (window, _events) = glfw
            .create_window(
                width,
                height,
                title.unwrap_or("Just Another Damn Engine"),
                mode.unwrap_or(WindowMode::Windowed),
            )
            .ok_or_else(|| glfw::InitError::Internal)?;

        Ok(Self {
            width,
            height,
            title: title.unwrap_or("Just Another Damn Engine").to_string(),
            instance: glfw,
            handle: window,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn handle(&mut self) -> &mut PWindow {
        &mut self.handle
    }

    pub fn instance(&mut self) -> &mut Glfw {
        &mut self.instance
    }

    pub fn should_close(&mut self) -> bool {
        self.handle.should_close()
    }

    pub fn poll_events(&mut self) {
        self.instance.poll_events();
    }

    pub fn swap_buffers(&mut self) {
        self.handle.swap_buffers();
    }
}
