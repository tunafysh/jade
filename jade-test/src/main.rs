use anyhow::Result;
use jade::{backend::{backend::Backend, opengl::backend::OpenGLBackend, window::Window}, util::types::BackendType};
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let window = Window::new(BackendType::OpenGL, 1280, 720, Some("test"), None)?;
    let mut backend = OpenGLBackend::init(window)?;

    while !backend.window().should_close() {
        backend.window().poll_events();
        backend.begin_frame();
        backend.end_frame();
    }
    Ok(())
}
