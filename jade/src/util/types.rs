#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BackendType {
    OpenGL,
    Vulkan,
    WebGPU,
}
