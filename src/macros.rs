#[macro_export]
macro_rules! skip_err {
    ($result:expr) => {
        match $result {
            Ok(value) => Ok(value),
            Err(_) => continue,
        }
    };
}
