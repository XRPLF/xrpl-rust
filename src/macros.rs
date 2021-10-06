#[macro_export]
macro_rules! skip_err {
    ($result:expr) => {
        match $result {
            Ok(value) => Ok(value),
            Err(_) => continue,
        }
    };
}

#[macro_export]
macro_rules! to_bytes {
    ($val:expr) => {
        if cfg!(target_endian = "big") {
            $val.to_be_bytes()
        } else {
            $val.to_le_bytes()
        }
    };
}
