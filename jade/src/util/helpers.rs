#[macro_export]
macro_rules! vertex {
    ($x:expr, $y:expr, $z:expr) => {
        $crate::graphics::vertex::Vertex {
            position: $crate::glam::Vec3::new($x, $y, $z),
            ..Default::default()
        }
    };
}
