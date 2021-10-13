/// General XRPL Model Exception.

#[derive(Debug)]
pub struct XRPLModelException(Cow<'static, str>);
