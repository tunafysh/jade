use anyhow::Result;
use jade::{
    backend::{backend::Backend, opengl::backend::OpenGLBackend, window::Window},
    shading::compiler::{Source, compile},
    util::types::BackendType,
};
use tracing::info;
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let shader = include_str!("../../assets/default.wgsl");

    let window = Window::new(BackendType::OpenGL, 1280, 720, Some("test"), None)?;
    let mut backend = OpenGLBackend::init(window)?;

    info!("Compiling sample shader to GLSL");

    let shader_source = Source::Source(shader.to_string());

    let (glsl_shader, glsl_reflection) = compile(shader_source.clone(), BackendType::OpenGL)?;
    info!(
        "GLSL Compiled Shader: {:#?}. GLSL Reflection: {:#?}.",
        glsl_shader, glsl_reflection
    );
    //let (spv_shader, spv_reflection) = compile(shader_source.clone(), BackendType::Vulkan)?;
    //info!("SPIR-V Compiled Shader: {:#?}. SPIR-V Reflection: {:#?}.", spv_shader, spv_reflection);
    let (wgsl_shader, wgsl_reflection) = compile(shader_source, BackendType::WebGPU)?;
    info!(
        "WGSL Compiled Shader: {:#?}. WGSL Reflection: {:#?}.",
        wgsl_shader, wgsl_reflection
    );

    while !backend.window().should_close() {
        backend.window().poll_events();
        backend.begin_frame()?;
        backend.end_frame();
    }
    Ok(())
}
