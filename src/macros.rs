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
macro_rules! mutate_from_error {
    ($name:ty, $except:ident) => {
        impl From<$name> for $except {
            fn from(err: $name) -> Self {
                let typed = alloc::any::type_name::<$name>();
                $except {
                    message: alloc::format!("{}: {}", typed, &err.to_string()),
                }
            }
        }
    };
}
