use crate::models::transactions::{AccountSetFlag, PaymentFlag};
use alloc::borrow::Cow;
use core::fmt::Debug;
use strum_macros::Display;
use thiserror_no_std::Error;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum XRPLTransactionException<'a> {
    XRPLAccountSetError(XRPLAccountSetException<'a>),
    XRPLCheckCashError(XRPLCheckCashException<'a>),
    XRPLDepositPreauthError(XRPLDepositPreauthException<'a>),
    XRPLEscrowCreateError(XRPLEscrowCreateException<'a>),
    XRPLEscrowFinishError(XRPLEscrowFinishException<'a>),
    XRPLNFTokenAcceptOfferError(XRPLNFTokenAcceptOfferException<'a>),
    XRPLNFTokenCancelOfferError(XRPLNFTokenCancelOfferException<'a>),
    XRPLNFTokenCreateOfferError(XRPLNFTokenCreateOfferException<'a>),
    XRPLNFTokenMintError(XRPLNFTokenMintException<'a>),
    XRPLPaymentError(XRPLPaymentException<'a>),
    XRPLSignerListSetError(XRPLSignerListSetException<'a>),
    TxMustBeSigned,
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLTransactionException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLTransactionFieldException<'a> {
    #[error("Transaction is missing common field `{0:?}`")]
    FieldMissing(&'a str),
    #[error("There is no transaction common field `{0:?}`")]
    InvalidCommonField(&'a str),
    #[error("There is no account field named `{0:?}`")]
    UnknownAccountField(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLAccountSetException<'a> {
    /// A fields value exceeds its maximum value.
    #[error("The value of the field `{field:?}` is defined above its maximum (max {max:?}, found {found:?}). For more information see: {resource:?}")]
    ValueTooHigh {
        field: Cow<'a, str>,
        max: u32,
        found: u32,
        resource: Cow<'a, str>,
    },
    /// A fields value exceeds its minimum value.
    #[error("The value of the field `{field:?}` is defined below its minimum (min {min:?}, found {found:?}). For more information see: {resource:?}")]
    ValueTooLow {
        field: Cow<'a, str>,
        min: u32,
        found: u32,
        resource: Cow<'a, str>,
    },
    /// A fields value exceeds its maximum character length.
    #[error("The value of the field `{field:?}` exceeds its maximum length of characters (max {max:?}, found {found:?}). For more information see: {resource:?}")]
    ValueTooLong {
        field: Cow<'a, str>,
        max: usize,
        found: usize,
        resource: Cow<'a, str>,
    },
    /// A fields value doesn't match its required format.
    #[error("The value of the field `{field:?}` does not have the correct format (expected {format:?}, found {found:?}). For more information see: {resource:?}")]
    InvalidValueFormat {
        field: Cow<'a, str>,
        format: Cow<'a, str>,
        found: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A field can only be defined if a transaction flag is set.
    #[error("For the field `{field:?}` to be defined it is required to set the flag `{flag:?}`. For more information see: {resource:?}")]
    FieldRequiresFlag {
        field: Cow<'a, str>,
        flag: AccountSetFlag,
        resource: Cow<'a, str>,
    },
    /// An account set flag can only be set if a field is defined.
    #[error("For the flag `{flag:?}` to be set it is required to define the field `{field:?}`. For more information see: {resource:?}")]
    FlagRequiresField {
        flag: AccountSetFlag,
        field: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// Am account set flag can not be set and unset at the same time.
    #[error("A flag cannot be set and unset at the same time (found {found:?}). For more information see: {resource:?}")]
    SetAndUnsetSameFlag {
        found: AccountSetFlag,
        resource: Cow<'a, str>,
    },
    /// A field was defined and an account set flag that is required for that field was unset.
    #[error("The field `{field:?}` cannot be defined if its required flag `{flag:?}` is being unset. For more information see: {resource:?}")]
    SetFieldWhenUnsetRequiredFlag {
        field: Cow<'a, str>,
        flag: AccountSetFlag,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLAccountSetException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLCheckCashException<'a> {
    /// A field cannot be defined with other fields.
    #[error("The field `{field1:?}` can not be defined with `{field2:?}`. Define exactly one of them. For more information see: {resource:?}")]
    DefineExactlyOneOf {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLDepositPreauthException<'a> {
    /// A field cannot be defined with other fields.
    #[error("The field `{field1:?}` can not be defined with `{field2:?}`. Define exactly one of them. For more information see: {resource:?}")]
    DefineExactlyOneOf {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLCheckCashException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLEscrowCreateException<'a> {
    /// A fields value cannot be below another fields value.
    #[error("The value of the field `{field1:?}` is not allowed to be below the value of the field `{field2:?}` (max {field2_val:?}, found {field1_val:?}). For more information see: {resource:?}")]
    ValueBelowValue {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        field1_val: u32,
        field2_val: u32,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLEscrowCreateException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLEscrowFinishException<'a> {
    /// For a field to be defined it also needs another field to be defined.
    #[error("For the field `{field1:?}` to be defined it is required to also define the field `{field2:?}`. For more information see: {resource:?}")]
    FieldRequiresField {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLEscrowFinishException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenAcceptOfferException<'a> {
    /// Define at least one of the fields.
    #[error("Define at least one of the fields `{field1:?}` and `{field2:?}`. For more information see: {resource:?}")]
    DefineOneOf {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// The value can not be zero.
    #[error("The value of the field `{field:?}` is not allowed to be zero. For more information see: {resource:?}")]
    ValueZero {
        field: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLNFTokenAcceptOfferException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenCancelOfferException<'a> {
    /// A collection was defined to be empty.
    #[error("The value of the field `{field:?}` is not allowed to be empty (type `{r#type:?}`). If the field is optional, define it to be `None`. For more information see: {resource:?}")]
    CollectionEmpty {
        field: Cow<'a, str>,
        r#type: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLNFTokenCancelOfferException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenCreateOfferException<'a> {
    /// The value can not be zero.
    #[error("The value of the field `{field:?}` is not allowed to be zero. For more information see: {resource:?}")]
    ValueZero {
        field: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A fields value is not allowed to be the same as another fields value.
    #[error("The value of the field `{field1:?}` is not allowed to be the same as the value of the field `{field2:?}`. For more information see: {resource:?}")]
    ValueEqualsValue {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// An optional value must be defined in a certain context.
    #[error("The optional field `{field:?}` is required to be defined for {context:?}. For more information see: {resource:?}")]
    OptionRequired {
        field: Cow<'a, str>,
        context: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// An optional value is not allowed to be defined in a certain context.
    #[error("The optional field `{field:?}` is not allowed to be defined for {context:?}. For more information see: {resource:?}")]
    IllegalOption {
        field: Cow<'a, str>,
        context: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLNFTokenCreateOfferException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenMintException<'a> {
    /// A fields value is not allowed to be the same as another fields value.
    #[error("The value of the field `{field1:?}` is not allowed to be the same as the value of the field `{field2:?}`. For more information see: {resource:?}")]
    ValueEqualsValue {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A fields value exceeds its maximum value.
    #[error("The field `{field:?}` exceeds its maximum value (max {max:?}, found {found:?}). For more information see: {resource:?}")]
    ValueTooHigh {
        field: Cow<'a, str>,
        max: u32,
        found: u32,
        resource: Cow<'a, str>,
    },
    /// A fields value exceeds its maximum character length.
    #[error("The value of the field `{field:?}` exceeds its maximum length of characters (max {max:?}, found {found:?}). For more information see: {resource:?}")]
    ValueTooLong {
        field: Cow<'a, str>,
        max: usize,
        found: usize,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLNFTokenMintException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLPaymentException<'a> {
    /// An optional value must be defined in a certain context.
    #[error("The optional field `{field:?}` is required to be defined for {context:?}. For more information see: {resource:?}")]
    OptionRequired {
        field: Cow<'a, str>,
        context: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// An optional value is not allowed to be defined in a certain context.
    #[error("The optional field `{field:?}` is not allowed to be defined for {context:?}.For more information see: {resource:?}")]
    IllegalOption {
        field: Cow<'a, str>,
        context: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A fields value is not allowed to be the same as another fields value, in a certain context.
    #[error("The value of the field `{field1:?}` is not allowed to be the same as the value of the field `{field2:?}`, for {context:?}. For more information see: {resource:?}")]
    ValueEqualsValueInContext {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        context: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// An account set flag can only be set if a field is defined.
    #[error("For the flag `{flag:?}` to be set it is required to define the field `{field:?}`. For more information see: {resource:?}")]
    FlagRequiresField {
        flag: PaymentFlag,
        field: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLPaymentException<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLSignerListSetException<'a> {
    /// A field was defined that another field definition would delete.
    #[error("The value of the field `{field1:?}` can not be defined with the field `{field2:?}` because it would cause the deletion of `{field1:?}`. For more information see: {resource:?}")]
    ValueCausesValueDeletion {
        field1: Cow<'a, str>,
        field2: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A field is expected to have a certain value to be deleted.
    #[error("The field `{field:?}` has the wrong value to be deleted (expected {expected:?}, found {found:?}). For more information see: {resource:?}")]
    InvalidValueForValueDeletion {
        field: Cow<'a, str>,
        expected: u32,
        found: u32,
        resource: Cow<'a, str>,
    },
    /// A collection has too few items in it.
    #[error("The value of the field `{field:?}` has too few items in it (min {min:?}, found {found:?}). For more information see: {resource:?}")]
    CollectionTooFewItems {
        field: Cow<'a, str>,
        min: usize,
        found: usize,
        resource: Cow<'a, str>,
    },
    /// A collection has too many items in it.
    #[error("The value of the field `{field:?}` has too many items in it (max {max:?}, found {found:?}). For more information see: {resource:?}")]
    CollectionTooManyItems {
        field: Cow<'a, str>,
        max: usize,
        found: usize,
        resource: Cow<'a, str>,
    },
    /// A collection is not allowed to have duplicates in it.
    #[error("The value of the field `{field:?}` has a duplicate in it (found {found:?}). For more information see: {resource:?}")]
    CollectionItemDuplicate {
        field: Cow<'a, str>,
        found: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    /// A collection contains an invalid value.
    #[error("The field `{field:?}` contains an invalid value (found {found:?}). For more information see: {resource:?}")]
    CollectionInvalidItem {
        field: Cow<'a, str>,
        found: Cow<'a, str>,
        resource: Cow<'a, str>,
    },
    #[error("The field `signer_quorum` must be below or equal to the sum of `signer_weight` in `signer_entries`. For more information see: {resource:?}")]
    SignerQuorumExceedsSignerWeight {
        max: u32,
        found: u32,
        resource: Cow<'a, str>,
    },
}

#[cfg(feature = "std")]
impl<'a> alloc::error::Error for XRPLSignerListSetException<'a> {}
