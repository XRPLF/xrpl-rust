use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// See Unsubscribe:
/// `<https://xrpl.org/unsubscribe.html>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unsubscribe<'a> {
    /// Keep the lifetime parameter consistent with other result types
    #[serde(skip)]
    phantom: PhantomData<&'a ()>,
}
