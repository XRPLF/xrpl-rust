use xrpl::utils::{posix_to_ripple_time, ripple_time_to_posix};

#[test]
fn it_converts_posix_to_ripple_time() {
    assert_eq!(posix_to_ripple_time(1660187459), Ok(713502659_i64));
}

#[test]
fn it_converts_ripple_time_to_posix() {
    assert_eq!(ripple_time_to_posix(713502659), Ok(1660187459));
}
