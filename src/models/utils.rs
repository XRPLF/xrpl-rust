//! Helper util functions for the models module.
use crate::models::exceptions::JSONRPCException;
use alloc::string::String;
use serde::{Deserialize, Serialize};

/// JSONRPC Request
#[derive(Debug, Clone, Serialize)]
pub struct Request<T: Send + Sync> {
    pub method: String,
    pub params: Option<T>,
    pub id: serde_json::Value,
    pub jsonrpc: Option<String>,
}

/// JSONRPC Response
#[derive(Debug, Clone, Deserialize)]
pub struct Response<T: Send + Sync> {
    pub id: serde_json::Value,
    pub result: Option<T>,
    pub error: Option<JSONRPCException>,
    pub jsonrpc: Option<String>,
}
