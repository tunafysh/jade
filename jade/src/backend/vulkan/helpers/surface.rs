use crate::backend::window::Window;
use anyhow::Result;
use ash::{
    Entry, Instance,
    vk::{Handle, SurfaceKHR},
};
use tracing::{info, instrument};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

#[instrument(name = "Vulkan - Create Surface", skip_all)]
pub fn create_surface(
    window: &mut Window,
    entry: &Entry,
    instance: &mut Instance,
) -> Result<SurfaceKHR> {
    let display_handle = window.handle().display_handle()?.as_raw();
    let window_handle = window.handle().window_handle()?.as_raw();
    let surface = unsafe {
        ash_window::create_surface(entry, instance, display_handle, window_handle, None)
    }?;
    info!(surface = surface.as_raw(), "Vulkan window surface created");

    Ok(surface)
}
