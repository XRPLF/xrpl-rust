use alloc::string::ToString;
use serde_json::Value;

use crate::core::{
    types::{AccountId, Currency},
    BinaryParser, Parser,
};

use super::{exceptions::XRPLTypeException, SerializedType, TryFromParser};

pub struct Issue(SerializedType);

impl TryFromParser for Issue {
    type Error = XRPLTypeException;

    fn from_parser(parser: &mut BinaryParser, length: Option<usize>) -> Result<Self, Self::Error> {
        let currency = Currency::from_parser(parser, length)?;
        let mut currency_bytes = currency.as_ref().to_vec();
        if currency.to_string() == "XRP" {
            Ok(Issue(SerializedType::from(currency_bytes)))
        } else {
            let issuer = parser.read(20)?;
            currency_bytes.extend_from_slice(&issuer);

            Ok(Issue(SerializedType::from(currency_bytes)))
        }
    }
}

impl TryFrom<Value> for Issue {
    type Error = XRPLTypeException;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.get("currency") == Some(&Value::String("XRP".to_string())) {
            let currency = Currency::try_from("XRP")?;
            Ok(Issue(SerializedType::from(currency.as_ref().to_vec())))
        } else if let Some(issued_currency) = value.as_object() {
            let cur = issued_currency["currency"]
                .as_str()
                .ok_or(XRPLTypeException::MissingField("currency"))?;
            let currency = Currency::try_from(cur)?;
            let issuer = issued_currency["issuer"]
                .as_str()
                .ok_or(XRPLTypeException::MissingField("issuer"))?;
            let account = AccountId::try_from(issuer)?;
            let mut currency_bytes = currency.as_ref().to_vec();
            currency_bytes.extend_from_slice(account.as_ref());

            Ok(Issue(SerializedType::from(currency_bytes)))
        } else {
            Err(XRPLTypeException::UnexpectedJSONType)
        }
    }
}

impl AsRef<[u8]> for Issue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
