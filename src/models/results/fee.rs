use serde::{Deserialize, Serialize};

use crate::models::amount::XRPAmount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee<'a> {
    pub drops: Drops<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drops<'a> {
    pub base_fee: XRPAmount<'a>,
    pub median_fee: XRPAmount<'a>,
    pub minimum_fee: XRPAmount<'a>,
    pub open_ledger_fee: XRPAmount<'a>,
}
