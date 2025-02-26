use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

/// Response format for the deposit_authorized method, which indicates whether
/// one account is authorized to send payments directly to another.
///
/// See Deposit Authorized:
/// `<https://xrpl.org/deposit_authorized.html>`
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DepositAuthorized<'a> {
    /// The credentials specified in the request, if any.
    pub credentials: Option<Cow<'a, [Cow<'a, str>]>>,
    /// Whether the specified source account is authorized to send payments
    /// directly to the destination account. If true, either the destination
    /// account does not require deposit authorization or the source account
    /// is preauthorized.
    pub deposit_authorized: bool,
    /// The destination account specified in the request.
    pub destination_account: Cow<'a, str>,
    /// The identifying hash of the ledger that was used to generate this
    /// response.
    pub ledger_hash: Option<Cow<'a, str>>,
    /// The ledger index of the ledger version that was used to generate
    /// this response.
    pub ledger_index: Option<u32>,
    /// The ledger index of the current in-progress ledger version, which
    /// was used to generate this response.
    pub ledger_current_index: Option<u32>,
    /// The source account specified in the request.
    pub source_account: Cow<'a, str>,
    /// If true, the information comes from a validated ledger version.
    pub validated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_authorized_deserialize() {
        let json = r#"{
            "credentials": [
                "A182EFBD154C9E80195082F86C1C8952FC0760A654B886F61BB0A59803B4387B",
                "383D269D6C7417D0A8716B09F5DB329FB17B45A5EFDBAFB82FF04BC420DCF7D5"
            ],
            "deposit_authorized": true,
            "destination_account": "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8",
            "ledger_hash": "BD03A10653ED9D77DCA859B7A735BF0580088A8F287FA2C5403E0A19C58EF322",
            "ledger_index": 8,
            "source_account": "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de",
            "validated": true
        }"#;

        let result: DepositAuthorized = serde_json::from_str(json).unwrap();

        assert_eq!(result.credentials.as_ref().unwrap().len(), 2);
        assert_eq!(
            result.credentials.as_ref().unwrap()[0],
            "A182EFBD154C9E80195082F86C1C8952FC0760A654B886F61BB0A59803B4387B"
        );
        assert_eq!(
            result.credentials.as_ref().unwrap()[1],
            "383D269D6C7417D0A8716B09F5DB329FB17B45A5EFDBAFB82FF04BC420DCF7D5"
        );
        assert!(result.deposit_authorized);
        assert_eq!(
            result.destination_account,
            "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8"
        );
        assert_eq!(
            result.ledger_hash.unwrap(),
            "BD03A10653ED9D77DCA859B7A735BF0580088A8F287FA2C5403E0A19C58EF322"
        );
        assert_eq!(result.ledger_index.unwrap(), 8);
        assert_eq!(result.source_account, "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de");
        assert!(result.validated.unwrap());
    }

    #[test]
    fn test_deposit_authorized_serialize() {
        let auth = DepositAuthorized {
            credentials: Some(
                alloc::vec![
                    "A182EFBD154C9E80195082F86C1C8952FC0760A654B886F61BB0A59803B4387B".into(),
                    "383D269D6C7417D0A8716B09F5DB329FB17B45A5EFDBAFB82FF04BC420DCF7D5".into(),
                ]
                .into(),
            ),
            deposit_authorized: true,
            destination_account: "rsUiUMpnrgxQp24dJYZDhmV4bE3aBtQyt8".into(),
            ledger_hash: Some(
                "BD03A10653ED9D77DCA859B7A735BF0580088A8F287FA2C5403E0A19C58EF322".into(),
            ),
            ledger_index: Some(8),
            ledger_current_index: None,
            source_account: "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de".into(),
            validated: Some(true),
        };

        let serialized = serde_json::to_string(&auth).unwrap();
        let deserialized: DepositAuthorized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(auth, deserialized);
    }
}
