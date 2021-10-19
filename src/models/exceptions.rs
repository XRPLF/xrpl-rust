/// General XRPL Model Exception.

#[derive(Debug)]
pub enum XRPLModelException {}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLModelException {}
