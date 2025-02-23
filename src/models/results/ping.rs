use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// See Ping:
/// `<https://xrpl.org/ping.html#ping>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ping<'a> {
    /// Keep the lifetime parameter consistent with other result types
    #[serde(skip)]
    phantom: PhantomData<&'a ()>,
}
