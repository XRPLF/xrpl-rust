use crate::{
    core::exceptions::XRPLCoreException,
    models::transactions::{account_set::AccountSetFlag, payment::PaymentFlag},
};
use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Debug, PartialEq, Error)]
pub enum XRPLTransactionException {
    #[error("{0}")]
    XRPLAccountSetError(#[from] XRPLAccountSetException),
    #[error("{0}")]
    XRPLNFTokenCancelOfferError(#[from] XRPLNFTokenCancelOfferException),
    #[error("{0}")]
    XRPLNFTokenCreateOfferError(#[from] XRPLNFTokenCreateOfferException),
    #[error("{0}")]
    XRPLPaymentError(#[from] XRPLPaymentException),
    #[error("{0}")]
    XRPLSignerListSetError(#[from] XRPLSignerListSetException),
    #[error("{0}")]
    XRPLXChainClaimError(#[from] XRPLXChainClaimException),
    #[error("{0}")]
    XRPLXChainCreateBridgeError(#[from] XRPLXChainCreateBridgeException),
    #[error("{0}")]
    XRPLXChainCreateClaimIDError(#[from] XRPLXChainCreateClaimIDException),
    #[error("{0}")]
    XRPLXChainModifyBridgeError(#[from] XRPLXChainModifyBridgeException),
    #[error("{0}")]
    XRPLCoreError(#[from] XRPLCoreException),
    #[error("The transaction must be signed")]
    TxMustBeSigned,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLTransactionException {}

#[derive(Debug, PartialEq, Error)]
pub enum XRPLTransactionFieldException {
    #[error("There is no transaction common field `{0:?}`")]
    InvalidCommonField(String),
    #[error("There is no account field named `{0:?}`")]
    UnknownAccountField(String),
}

#[derive(Debug, PartialEq, Error)]
pub enum XRPLAccountSetException {
    /// A field can only be defined if a transaction flag is set.
    #[error("For the field `{field:?}` to be defined it is required to set the flag `{flag:?}`")]
    FieldRequiresFlag { field: String, flag: AccountSetFlag },
    /// An account set flag can only be set if a field is defined.
    #[error("For the flag `{flag:?}` to be set it is required to define the field `{field:?}`")]
    FlagRequiresField { flag: AccountSetFlag, field: String },
    /// Am account set flag can not be set and unset at the same time.
    #[error("A flag cannot be set and unset at the same time (found {found:?})")]
    SetAndUnsetSameFlag { found: AccountSetFlag },
    /// A field was defined and an account set flag that is required for that field was unset.
    #[error(
        "The field `{field:?}` cannot be defined if its required flag `{flag:?}` is being unset"
    )]
    SetFieldWhenUnsetRequiredFlag { field: String, flag: AccountSetFlag },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLAccountSetException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenCancelOfferException {
    /// A collection was defined to be empty.
    #[error("The value of the field `{field:?}` is not allowed to be empty (type `{r#type:?}`). If the field is optional, define it to be `None`")]
    CollectionEmpty { field: String, r#type: String },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLNFTokenCancelOfferException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLNFTokenCreateOfferException {
    /// An optional value must be defined in a certain context.
    #[error("The optional field `{field:?}` is required to be defined for {context:?}")]
    OptionRequired { field: String, context: String },
    /// An optional value is not allowed to be defined in a certain context.
    #[error("The optional field `{field:?}` is not allowed to be defined for {context:?}")]
    IllegalOption { field: String, context: String },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLNFTokenCreateOfferException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum XRPLPaymentException {
    /// An optional value must be defined in a certain context.
    #[error("The optional field `{field:?}` is required to be defined for {context:?}")]
    OptionRequired { field: String, context: String },
    /// An optional value is not allowed to be defined in a certain context.
    #[error("The optional field `{field:?}` is not allowed to be defined for {context:?}")]
    IllegalOption { field: String, context: String },
    /// A fields value is not allowed to be the same as another fields value, in a certain context.
    #[error("The value of the field `{field1:?}` is not allowed to be the same as the value of the field `{field2:?}`, for {context:?}")]
    ValueEqualsValueInContext {
        field1: String,
        field2: String,
        context: String,
    },
    /// An account set flag can only be set if a field is defined.
    #[error("For the flag `{flag:?}` to be set it is required to define the field `{field:?}`")]
    FlagRequiresField { flag: PaymentFlag, field: String },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLPaymentException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLSignerListSetException {
    /// A field was defined that another field definition would delete.
    #[error("The value of the field `{field1:?}` can not be defined with the field `{field2:?}` because it would cause the deletion of `{field1:?}`")]
    ValueCausesValueDeletion { field1: String, field2: String },
    /// A field is expected to have a certain value to be deleted.
    #[error("The field `{field:?}` has the wrong value to be deleted (expected {expected:?}, found {found:?})")]
    InvalidValueForValueDeletion {
        field: String,
        expected: u32,
        found: u32,
    },
    /// A collection has too few items in it.
    #[error(
        "The value of the field `{field:?}` has too few items in it (min {min:?}, found {found:?})"
    )]
    CollectionTooFewItems {
        field: String,
        min: usize,
        found: usize,
    },
    /// A collection has too many items in it.
    #[error("The value of the field `{field:?}` has too many items in it (max {max:?}, found {found:?})")]
    CollectionTooManyItems {
        field: String,
        max: usize,
        found: usize,
    },
    /// A collection is not allowed to have duplicates in it.
    #[error("The value of the field `{field:?}` has a duplicate in it (found {found:?})")]
    CollectionItemDuplicate { field: String, found: String },
    /// A collection contains an invalid value.
    #[error("The field `{field:?}` contains an invalid value (found {found:?})")]
    CollectionInvalidItem { field: String, found: String },
    #[error("The field `signer_quorum` must be below or equal to the sum of `signer_weight` in `signer_entries`")]
    SignerQuorumExceedsSignerWeight { max: u32, found: u32 },
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLSignerListSetException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLXChainClaimException {
    #[error("`amount` must match either `locking_chain_issue` or `issuing_chain_issue`")]
    AmountMismatch,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLXChainClaimException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLXChainCreateBridgeException {
    #[error("Cannot have the same door accounts on the locking and issuing chain")]
    SameDoorAccounts,
    #[error(
        "The `account` field must either match the `locking_chain_door` or `issuing_chain_door`"
    )]
    AccountDoorMismatch,
    #[error("Bridge must be XRP-XRP or IOU-IOU")]
    CrossCurrencyBridgeNotAllowed,
    #[error("Cannot have MinAccountCreateAmount if bridge is IOU-IOU")]
    MinAccountCreateAmountForIOU,
    #[error("`signature_reward` must be numeric")]
    SignatureRewartMustBeNumberic,
    #[error("`min_account_create_amount` must be numeric")]
    MinAccountCreateAmountMustBeNumberic,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLXChainCreateBridgeException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLXChainCreateClaimIDException {
    #[error("`other_chain_source` must be a valid XRPL address")]
    OtherChainSourceIsInvalid,
}

#[cfg(feature = "std")]
impl alloc::error::Error for XRPLXChainCreateClaimIDException {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum XRPLXChainModifyBridgeException {
    #[error("Must either change `signature_reward`, change `min_account_create_amount`, or clear `min_account_create_amount`")]
    MustChangeOrClear,
    #[error("`account` must be either `locking_chain_door` or `issuing_chain_door`")]
    AccountDoorMismatch,
    #[error("Cannot have MinAccountCreateAmount if bridge is IOU-IOU")]
    CannotHaveMinAccountCreateAmount,
}
