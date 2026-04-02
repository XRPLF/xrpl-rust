//! Property-based tests for XLS-70 credential types.
//!
//! These tests exercise validation boundaries and serde round-trip correctness
//! for CredentialCreate, CredentialAccept, CredentialDelete, DepositPreauth,
//! and the shared `validate_credential_ids` helper using randomly generated
//! inputs via proptest.

use std::borrow::Cow;

use proptest::prelude::*;
use xrpl::models::transactions::credential_accept::CredentialAccept;
use xrpl::models::transactions::credential_create::CredentialCreate;
use xrpl::models::transactions::credential_delete::CredentialDelete;
use xrpl::models::transactions::deposit_preauth::DepositPreauth;
use xrpl::models::transactions::{CommonFields, TransactionType};
use xrpl::models::{
    CredentialAuthorization, CredentialAuthorizationFields, FlagCollection, Model, NoFlags,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A fixed valid-looking XRPL account address for use in test structs.
const ACCOUNT_A: &str = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb";
const ACCOUNT_B: &str = "rEhxGqkqPPSxQ3P25J66ft5TwpzV14k2de";
const ACCOUNT_C: &str = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";

/// Build minimal `CommonFields` for a given transaction type.
fn common_fields(account: &str, tt: TransactionType) -> CommonFields<'_, NoFlags> {
    CommonFields {
        account: Cow::Borrowed(account),
        transaction_type: tt,
        fee: Some("10".into()),
        flags: FlagCollection::default(),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// 1. CredentialType length property
//    Valid: 1..=128 hex chars.  Invalid: 0 or >128.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn credential_type_valid_length(len in 1_usize..=128) {
        let ct = "A".repeat(len);
        let tx = CredentialCreate {
            common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialCreate),
            subject: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
            expiration: None,
            uri: None,
        };
        prop_assert!(tx.get_errors().is_ok(), "len {} should be valid", len);
    }

    #[test]
    fn credential_type_too_long(extra in 1_usize..=200) {
        let len = 128 + extra;
        let ct = "A".repeat(len);
        let tx = CredentialCreate {
            common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialCreate),
            subject: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
            expiration: None,
            uri: None,
        };
        prop_assert!(tx.get_errors().is_err(), "len {} should be rejected", len);
    }
}

#[test]
fn credential_type_empty_is_rejected() {
    let tx = CredentialCreate {
        common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialCreate),
        subject: Cow::Borrowed(ACCOUNT_B),
        credential_type: Cow::Borrowed(""),
        expiration: None,
        uri: None,
    };
    assert!(tx.get_errors().is_err());
}

// Also test CredentialAccept uses the same boundary.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn credential_accept_type_valid(len in 1_usize..=128) {
        let ct = "B".repeat(len);
        let tx = CredentialAccept {
            common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialAccept),
            issuer: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
        };
        prop_assert!(tx.get_errors().is_ok());
    }

    #[test]
    fn credential_accept_type_too_long(extra in 1_usize..=200) {
        let len = 128 + extra;
        let ct = "B".repeat(len);
        let tx = CredentialAccept {
            common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialAccept),
            issuer: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
        };
        prop_assert!(tx.get_errors().is_err());
    }
}

// ---------------------------------------------------------------------------
// 2. CredentialDelete subject/issuer property
//    At least one of subject/issuer must be Some.
//    When both are provided, account must match one.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// When only subject is provided (issuer omitted, account implicitly fills
    /// the issuer role), validation passes regardless of the account value.
    #[test]
    fn credential_delete_subject_only_always_valid(
        // pick any of the three accounts as the submitter
        acct_idx in 0_usize..3,
    ) {
        let accounts = [ACCOUNT_A, ACCOUNT_B, ACCOUNT_C];
        let acct = accounts[acct_idx];
        let tx = CredentialDelete {
            common_fields: common_fields(acct, TransactionType::CredentialDelete),
            subject: Some(Cow::Borrowed(ACCOUNT_B)),
            issuer: None,
            credential_type: Cow::Borrowed("4B5943"),
        };
        prop_assert!(tx.get_errors().is_ok());
    }

    /// When only issuer is provided (subject omitted, account implicitly fills
    /// the subject role), validation passes regardless of the account value.
    #[test]
    fn credential_delete_issuer_only_always_valid(
        acct_idx in 0_usize..3,
    ) {
        let accounts = [ACCOUNT_A, ACCOUNT_B, ACCOUNT_C];
        let acct = accounts[acct_idx];
        let tx = CredentialDelete {
            common_fields: common_fields(acct, TransactionType::CredentialDelete),
            subject: None,
            issuer: Some(Cow::Borrowed(ACCOUNT_A)),
            credential_type: Cow::Borrowed("4B5943"),
        };
        prop_assert!(tx.get_errors().is_ok());
    }

    /// When both subject and issuer are provided, the account must equal one.
    #[test]
    fn credential_delete_both_account_must_match(
        use_subject in proptest::bool::ANY,
    ) {
        // account matches whichever field `use_subject` picks
        let acct = if use_subject { ACCOUNT_A } else { ACCOUNT_B };
        let tx = CredentialDelete {
            common_fields: common_fields(acct, TransactionType::CredentialDelete),
            subject: Some(Cow::Borrowed(ACCOUNT_A)),
            issuer: Some(Cow::Borrowed(ACCOUNT_B)),
            credential_type: Cow::Borrowed("4B5943"),
        };
        prop_assert!(tx.get_errors().is_ok());
    }
}

#[test]
fn credential_delete_none_none_fails() {
    let tx = CredentialDelete {
        common_fields: common_fields(ACCOUNT_A, TransactionType::CredentialDelete),
        subject: None,
        issuer: None,
        credential_type: Cow::Borrowed("4B5943"),
    };
    assert!(tx.get_errors().is_err());
}

#[test]
fn credential_delete_both_mismatch_fails() {
    // account (C) matches neither subject (A) nor issuer (B)
    let tx = CredentialDelete {
        common_fields: common_fields(ACCOUNT_C, TransactionType::CredentialDelete),
        subject: Some(Cow::Borrowed(ACCOUNT_A)),
        issuer: Some(Cow::Borrowed(ACCOUNT_B)),
        credential_type: Cow::Borrowed("4B5943"),
    };
    assert!(tx.get_errors().is_err());
}

// ---------------------------------------------------------------------------
// 3. CredentialIDs length property (via Payment, which calls validate_credential_ids)
//    None => Ok, 1..=8 => Ok, 0 => Err, >8 => Err.
// ---------------------------------------------------------------------------

// We test the public `validate_credential_ids` function indirectly through
// AccountDelete, which is the simplest transaction that calls it.

use xrpl::models::transactions::account_delete::AccountDelete;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn credential_ids_valid_length(count in 1_usize..=8) {
        let ids: Vec<Cow<'_, str>> = (0..count)
            .map(|i| Cow::Owned(format!("{:064X}", i)))
            .collect();
        let tx = AccountDelete {
            common_fields: common_fields(ACCOUNT_A, TransactionType::AccountDelete),
            destination: Cow::Borrowed(ACCOUNT_B),
            destination_tag: None,
            credential_ids: Some(ids),
        };
        prop_assert!(tx.get_errors().is_ok(), "count {} should be valid", count);
    }

    #[test]
    fn credential_ids_too_many(extra in 1_usize..=20) {
        let count = 8 + extra;
        let ids: Vec<Cow<'_, str>> = (0..count)
            .map(|i| Cow::Owned(format!("{:064X}", i)))
            .collect();
        let tx = AccountDelete {
            common_fields: common_fields(ACCOUNT_A, TransactionType::AccountDelete),
            destination: Cow::Borrowed(ACCOUNT_B),
            destination_tag: None,
            credential_ids: Some(ids),
        };
        prop_assert!(tx.get_errors().is_err(), "count {} should be rejected", count);
    }
}

#[test]
fn credential_ids_empty_is_rejected() {
    let tx = AccountDelete {
        common_fields: common_fields(ACCOUNT_A, TransactionType::AccountDelete),
        destination: Cow::Borrowed(ACCOUNT_B),
        destination_tag: None,
        credential_ids: Some(vec![]),
    };
    assert!(tx.get_errors().is_err());
}

#[test]
fn credential_ids_none_is_valid() {
    let tx = AccountDelete {
        common_fields: common_fields(ACCOUNT_A, TransactionType::AccountDelete),
        destination: Cow::Borrowed(ACCOUNT_B),
        destination_tag: None,
        credential_ids: None,
    };
    assert!(tx.get_errors().is_ok());
}

// ---------------------------------------------------------------------------
// 4. DepositPreauth exactly-one property
//    Exactly one of the four authorization fields must be Some.
// ---------------------------------------------------------------------------

/// Helper to build a CredentialAuthorization vec of the given size.
fn cred_auth_vec(n: usize) -> Vec<CredentialAuthorization<'static>> {
    (0..n)
        .map(|_| {
            CredentialAuthorization::new(CredentialAuthorizationFields::new(
                Cow::Borrowed(ACCOUNT_B),
                Cow::Borrowed("4B5943"),
            ))
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Exactly one of the four fields set => valid.
    #[test]
    fn deposit_preauth_exactly_one_valid(which in 0_u8..4) {
        let tx = DepositPreauth {
            common_fields: common_fields(ACCOUNT_A, TransactionType::DepositPreauth),
            authorize: if which == 0 { Some(Cow::Borrowed(ACCOUNT_B)) } else { None },
            unauthorize: if which == 1 { Some(Cow::Borrowed(ACCOUNT_B)) } else { None },
            authorize_credentials: if which == 2 { Some(cred_auth_vec(1)) } else { None },
            unauthorize_credentials: if which == 3 { Some(cred_auth_vec(1)) } else { None },
        };
        prop_assert!(
            tx.get_errors().is_ok(),
            "field index {} should be valid when it's the only one set",
            which
        );
    }

    /// Two or more fields set => invalid.
    #[test]
    fn deposit_preauth_multiple_fields_invalid(
        bits in 3_u8..=15, // at least 2 bits set (3 = 0b0011)
    ) {
        // Only test values with 2+ bits set
        let popcount = bits.count_ones();
        prop_assume!(popcount >= 2);

        let tx = DepositPreauth {
            common_fields: common_fields(ACCOUNT_A, TransactionType::DepositPreauth),
            authorize: if bits & 1 != 0 { Some(Cow::Borrowed(ACCOUNT_B)) } else { None },
            unauthorize: if bits & 2 != 0 { Some(Cow::Borrowed(ACCOUNT_C)) } else { None },
            authorize_credentials: if bits & 4 != 0 { Some(cred_auth_vec(1)) } else { None },
            unauthorize_credentials: if bits & 8 != 0 { Some(cred_auth_vec(1)) } else { None },
        };
        prop_assert!(
            tx.get_errors().is_err(),
            "bits={:04b} ({} fields set) should be invalid",
            bits,
            popcount
        );
    }
}

#[test]
fn deposit_preauth_none_set_fails() {
    let tx = DepositPreauth {
        common_fields: common_fields(ACCOUNT_A, TransactionType::DepositPreauth),
        authorize: None,
        unauthorize: None,
        authorize_credentials: None,
        unauthorize_credentials: None,
    };
    assert!(tx.get_errors().is_err());
}

// ---------------------------------------------------------------------------
// 5. Serde round-trip property
//    Serialize then deserialize produces the same struct.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn serde_roundtrip_credential_create(
        ct in "[0-9A-F]{2,128}",
        has_expiration in proptest::bool::ANY,
        expiration_val in proptest::num::u32::ANY,
        has_uri in proptest::bool::ANY,
        uri_hex in "[0-9A-F]{2,200}",
    ) {
        let tx = CredentialCreate {
            common_fields: CommonFields {
                account: Cow::Borrowed(ACCOUNT_A),
                transaction_type: TransactionType::CredentialCreate,
                fee: Some("12".into()),
                sequence: Some(42),
                signing_pub_key: Some(Cow::Borrowed("")),
                ..Default::default()
            },
            subject: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
            expiration: if has_expiration { Some(expiration_val) } else { None },
            uri: if has_uri { Some(Cow::Owned(uri_hex)) } else { None },
        };
        let json = serde_json::to_string(&tx).unwrap();
        let roundtripped: CredentialCreate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tx, &roundtripped);
    }

    #[test]
    fn serde_roundtrip_credential_delete(
        ct in "[0-9A-F]{2,64}",
        has_subject in proptest::bool::ANY,
        has_issuer in proptest::bool::ANY,
    ) {
        // Ensure at least one of subject/issuer is set so the struct is valid.
        let subject = if has_subject || !has_issuer {
            Some(Cow::Borrowed(ACCOUNT_A))
        } else {
            None
        };
        let issuer = if has_issuer {
            Some(Cow::Borrowed(ACCOUNT_B))
        } else {
            None
        };
        // When both are set, account must match one.
        let acct = if subject.is_some() {
            ACCOUNT_A
        } else {
            ACCOUNT_B
        };

        let tx = CredentialDelete {
            common_fields: CommonFields {
                account: Cow::Borrowed(acct),
                transaction_type: TransactionType::CredentialDelete,
                fee: Some("10".into()),
                sequence: Some(7),
                signing_pub_key: Some(Cow::Borrowed("")),
                ..Default::default()
            },
            subject,
            issuer,
            credential_type: Cow::Owned(ct),
        };

        // Verify it's valid before testing round-trip.
        prop_assert!(tx.get_errors().is_ok());

        let json = serde_json::to_string(&tx).unwrap();
        let roundtripped: CredentialDelete = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tx, &roundtripped);
    }

    #[test]
    fn serde_roundtrip_credential_accept(
        ct in "[0-9A-F]{2,128}",
    ) {
        let tx = CredentialAccept {
            common_fields: CommonFields {
                account: Cow::Borrowed(ACCOUNT_A),
                transaction_type: TransactionType::CredentialAccept,
                fee: Some("10".into()),
                sequence: Some(1),
                signing_pub_key: Some(Cow::Borrowed("")),
                ..Default::default()
            },
            issuer: Cow::Borrowed(ACCOUNT_B),
            credential_type: Cow::Owned(ct),
        };

        let json = serde_json::to_string(&tx).unwrap();
        let roundtripped: CredentialAccept = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tx, &roundtripped);
    }
}
