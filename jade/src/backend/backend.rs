use crate::shading::{reflection::ShaderReflection, shader::Shader};

use super::draw_command::DrawCommand;
use super::window::Window;
use anyhow::Result;

pub trait Backend {
    fn init(window: Window) -> Result<Self>
    where
        Self: Sized;

    fn add_shader(&mut self, shader: Shader, reflection: ShaderReflection);
    fn begin_frame(&mut self) -> Result<()>;
    fn draw(&mut self) -> Result<()>;
    fn end_frame(&mut self) -> Result<()>;
    fn window(&mut self) -> &mut Window;
    // no destroy. overload drop if needed.
}
