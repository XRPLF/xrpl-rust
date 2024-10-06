/// Turns a `thiserror_no_std::Error` into a `anyhow::Error`
#[macro_export]
macro_rules! Err {
    ($err:expr $(,)?) => {{
        use alloc::string::ToString;

        let error = $err.to_string().replace("\"", "");
        let boxed_error = ::alloc::boxed::Box::new(error);
        let leaked_error: &'static str = ::alloc::boxed::Box::leak(boxed_error);
        Err(anyhow::anyhow!(leaked_error))
    }};
}
