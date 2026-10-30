use tracing::info;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window as WinitWindow, WindowAttributes, WindowId},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum WindowMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    width: u32,
    height: u32,
    title: String,
    mode: WindowMode,
    resizable: bool,
}

impl WindowConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            title: "Just Another Damn Engine".into(),
            mode: WindowMode::Windowed,
            resizable: true,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn title_text(&self) -> &str {
        &self.title
    }

    pub fn window_mode(&self) -> WindowMode {
        self.mode
    }

    pub(crate) fn attributes(&self) -> WindowAttributes {
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .with_visible(true)
            .with_resizable(self.resizable);

        match self.mode {
            WindowMode::Windowed => attributes,
            WindowMode::BorderlessFullscreen => {
                attributes.with_fullscreen(Some(Fullscreen::Borderless(None)))
            }
        }
    }
}

pub struct Window {
    handle: WinitWindow,
    title: String,
}

impl Window {
    pub(crate) fn from_winit(handle: WinitWindow, title: String) -> Self {
        Self { handle, title }
    }

    pub fn create(
        event_loop: &ActiveEventLoop,
        config: &WindowConfig,
    ) -> Result<Self, winit::error::OsError> {
        let handle = event_loop.create_window(config.attributes())?;
        handle.set_visible(true);
        info!(window_id = ?handle.id(), width = handle.inner_size().width, height = handle.inner_size().height, "Native window created and shown");
        Ok(Self::from_winit(handle, config.title.clone()))
    }

    pub fn width(&self) -> u32 {
        self.handle.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.handle.inner_size().height
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.handle.inner_size()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();

        self.handle.set_title(&title);
        self.title = title;
    }

    pub fn id(&self) -> WindowId {
        self.handle.id()
    }

    pub fn request_redraw(&self) {
        self.handle.request_redraw();
    }

    pub fn show(&self) {
        self.handle.set_visible(true);
        info!(window_id = ?self.handle.id(), "Requested to show window.");
    }

    pub fn is_focused(&self) -> bool {
        self.handle.has_focus()
    }

    pub fn is_minimized(&self) -> Option<bool> {
        self.handle.is_minimized()
    }

    pub fn handle(&self) -> &WinitWindow {
        &self.handle
    }
}
