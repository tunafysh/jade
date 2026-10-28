#[macro_export]
macro_rules! drop_error {
    ($expr:expr) => {
        match $expr {
            Ok(()) => {}
            Err(err) => {
                tracing::error!(?err, "Cleanup operation failed");
            }
        }
    };
}
