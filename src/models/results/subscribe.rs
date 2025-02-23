use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// See Subscribe:
/// `<https://xrpl.org/subscribe.html#subscribe>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subscribe<'a> {
    /// Keep the lifetime parameter consistent with other result types
    #[serde(skip)]
    phantom: PhantomData<&'a ()>,
}
