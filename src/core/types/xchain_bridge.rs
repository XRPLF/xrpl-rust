use alloc::vec::Vec;
use serde::Deserialize;

use crate::core::{BinaryParser, Parser};

use super::{
    exceptions::XRPLTypeException, AccountId, Issue, SerializedType, TryFromParser, XRPLType,
};

const TYPE_ORDER: [[&str; 2]; 4] = [
    ["LockingChainDoor", "AccountID"],
    ["LockingChainIssue", "Issue"],
    ["IssuingChainDoor", "AccountID"],
    ["IssuingChainIssue", "Issue"],
];

#[derive(Debug, Deserialize, Clone)]
pub struct XChainBridge(SerializedType);

impl XRPLType for XChainBridge {
    type Error = XRPLTypeException;

    fn new(buffer: Option<&[u8]>) -> anyhow::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if let Some(buf) = buffer {
            Ok(XChainBridge(SerializedType::from(buf.to_vec())))
        } else {
            Ok(XChainBridge(SerializedType::from(Vec::new())))
        }
    }
}

impl TryFromParser for XChainBridge {
    type Error = XRPLTypeException;

    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Self, Self::Error> {
        let mut buf = Vec::new();
        for [_, object_type] in TYPE_ORDER {
            if object_type == "AccountID" {
                // skip the `14` byte and add it by hand
                let _ = parser.read(1);
                buf.extend_from_slice(hex::decode("14")?.as_ref());
            }
            match object_type {
                "AccountID" => {
                    let account_id = AccountId::from_parser(parser, length)?;
                    buf.extend_from_slice(account_id.as_ref());
                }
                "Issue" => {
                    let issue = Issue::from_parser(parser, length)?;
                    buf.extend_from_slice(issue.as_ref());
                }
                _ => unreachable!(),
            };
        }
        Ok(XChainBridge(SerializedType::from(buf)))
    }
}

impl TryFrom<&str> for XChainBridge {
    type Error = XRPLTypeException;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(XChainBridge(SerializedType::from(hex::decode(value)?)))
    }
}

impl AsRef<[u8]> for XChainBridge {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
