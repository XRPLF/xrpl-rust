use alloc::borrow::Cow;
use strum_macros::Display;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum XRPLRequestException<'a> {
    XRPLChannelAuthorizeError(XRPLChannelAuthorizeException<'a>),
    XRPLLedgerEntryError(XRPLLedgerEntryException<'a>),
    /*SignAndSubmitError(SignAndSubmitException),
    SignForError(SignForException),
    SignError(SignException),*/
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLRequestException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLChannelAuthorizeException<'a> {
    /// A field cannot be defined with other fields.
    #[error("The field `{field1:?}` can not be defined with `{field2:?}`, `{field3:?}`, `{field4:?}`. Define exactly one of them. For more information see: {resource:?}")]
    DefineExactlyOneOf {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        field3: Cow<'a, str>,
        field4: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

/*impl<'a> From<XRPLChannelAuthorizeException<'a>> for anyhow::Error {
    fn from(value: XRPLChannelAuthorizeException<'a>) -> Self {
        anyhow::anyhow!("{:?}", value)
    }
}*/

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLChannelAuthorizeException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLLedgerEntryException<'a> {
    /// A field cannot be defined with other fields.
    #[error("Define one of: `{field1:?}`, `{field2:?}`, `{field3:?}`, `{field4:?}`, `{field5:?}`, `{field6:?}`, `{field7:?}`, `{field8:?}`, `{field9:?}`, `{field10:?}`. Define exactly one of them. For more information see: {resource:?}")]
    DefineExactlyOneOf {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        field3: Cow<'a, str>,
        field4: Cow<'a, str>,
        field5: Cow<'a, str>,
        field6: Cow<'a, str>,
        field7: Cow<'a, str>,
        field8: Cow<'a, str>,
        field9: Cow<'a, str>,
        field10: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLLedgerEntryException<'a> {}

/*#[derive(Debug, Clone, PartialEq, Display)]
pub enum SignAndSubmitException {
    InvalidMustSetExactlyOneOf { fields: String },
    InvalidMustOmitKeyTypeIfSecretProvided,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum SignForException {
    InvalidMustSetExactlyOneOf { fields: String },
    InvalidMustOmitKeyTypeIfSecretProvided,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum SignException {
    InvalidMustSetExactlyOneOf { fields: String },
    InvalidMustOmitKeyTypeIfSecretProvided,
}*/
