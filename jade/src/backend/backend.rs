use super::draw_command::DrawCommand;
use anyhow::Result;
use super::window::Window;

pub trait Backend {
    fn init(window: Window) -> Result<Self>
    where
        Self: Sized;

        fn add_shader(&mut self);
        fn begin_frame(&mut self) -> Result<()>;
        fn draw(&mut self, command: &DrawCommand);
        fn end_frame(&mut self);
        fn window(&mut self) -> &mut Window;
    // no destroy. overload drop if needed.
}
