use alloc::borrow::Cow;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialAuthorizationFields<'a> {
    pub issuer: Cow<'a, str>,
    pub credential_type: Cow<'a, str>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, new)]
#[serde(rename_all = "PascalCase")]
pub struct CredentialAuthorization<'a> {
    pub credential: CredentialAuthorizationFields<'a>,
}
