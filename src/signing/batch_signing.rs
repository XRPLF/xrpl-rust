//! Helpers for signing multi-account Batch transactions (XLS-56) and combining
//! the BatchSigners collected from several parties.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::asynch::exceptions::XRPLHelperResult;
use crate::core::binarycodec::encode_for_signing_batch_v1_1;
use crate::core::keypairs::sign;
use crate::models::transactions::batch::{invalid, sort_by_account_id, Batch, BatchSigner};
use crate::models::transactions::exceptions::XRPLTransactionException;
use crate::models::transactions::Signer;
use crate::models::XRPLModelResult;
use crate::wallet::Wallet;

/// How `sign_multiaccount_batch` signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultisignMode<'m> {
    /// Single-sign (SigningPubKey + TxnSignature).
    Off,
    /// Multi-sign with the wallet's own address as the inner signer.
    WalletAddress,
    /// Multi-sign on behalf of this address (e.g. when signing with a regular key).
    Address(&'m str),
}

/// Sign a multi-account Batch. Returns a copy whose `BatchSigners` holds only
/// this signature; merge fragments with [`combine_batch_signers`].
///
/// `batch_account` is the account in the Batch this signature is for; it
/// defaults to the wallet's address. Pass it when the signing key differs from
/// that account (e.g. a regular key).
pub fn sign_multi_batch<'a>(
    wallet: &Wallet,
    batch: &Batch<'a>,
    multisign: MultisignMode<'_>,
    batch_account: Option<&str>,
) -> XRPLHelperResult<Batch<'a>> {
    let signing_account = batch_account.unwrap_or(&wallet.classic_address);
    let multisign_address: Option<&str> = match multisign {
        MultisignMode::Off => None,
        MultisignMode::WalletAddress => Some(&wallet.classic_address),
        MultisignMode::Address(address) => Some(address),
    };

    // An account can't be on its own signer list (rippled: "Invalid multisigner").
    if multisign_address == Some(signing_account) {
        return Err(invalid(
            "batch_signers",
            "a multi-signed signer account different from the batch account",
            signing_account,
        )
        .into());
    }

    // An account must sign if it authorizes an inner transaction (its delegate
    // when present, else its account), is the counterparty of one
    if !batch
        .involved_accounts()
        .iter()
        .any(|a| a == signing_account)
    {
        return Err(invalid(
            "batch_account",
            "an account included in the Batch",
            signing_account,
        )
        .into());
    }

    let tx_ids = batch
        .raw_transactions
        .iter()
        .map(|v| v.transaction_hash())
        .collect::<XRPLModelResult<Vec<String>>>()?;

    let mut seen = HashSet::with_capacity(tx_ids.len());
    let mut dups = HashSet::new();

    for s in &tx_ids {
        if !seen.insert(s.as_str()) {
            dups.insert(s.as_str());
        }
    }

    if !dups.is_empty() {
        Err(invalid(
            "raw_transactions",
            "no duplicate transactions", // temREDUNDANT
            "duplicate",
        ))?;
    }

    let signature = sign_batch_payload(wallet, batch, signing_account, multisign_address)?;
    let batch_signer =
        construct_batch_signer(signing_account, wallet, signature, multisign_address)?;

    let mut signed = batch.clone();
    signed.batch_signers = Some(alloc::vec![batch_signer]);

    Ok(signed)
}

/// Signs the Batch signing payload for `batch_account`. Multi-signed entries
/// also bind the inner signer account (`multisign_address`).
fn sign_batch_payload(
    wallet: &Wallet,
    batch: &Batch<'_>,
    batch_account: &str,
    multisign_address: Option<&str>,
) -> XRPLHelperResult<String> {
    let payload = hex::decode(batch_signing_data(batch, batch_account, multisign_address)?)?;

    Ok(sign(&payload, &wallet.private_key)?)
}

/// Hex payload a BatchSigner signs. `batch_account` is the `BatchSigner.Account`;
/// `multisigner_account` is the individual signer for multi-signed entries.
/// Requires the outer `Sequence`/`TicketSequence` and flags to be final.
pub fn batch_signing_data(
    batch: &Batch<'_>,
    batch_account: &str,
    multisign_address: Option<&str>,
) -> XRPLModelResult<String> {
    let ids = batch
        .raw_transactions
        .iter()
        .map(|v| v.transaction_hash())
        .collect::<XRPLModelResult<Vec<String>>>()?;

    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();

    Ok(encode_for_signing_batch_v1_1(
        &batch.common_fields.account,
        batch.outer_sequence()?,
        batch.mode_flags(),
        &id_refs,
        batch_account,
        multisign_address,
    )
    .map_err(XRPLTransactionException::XRPLCoreError)?)
}

fn construct_batch_signer(
    batch_account: &str,
    wallet: &Wallet,
    signature: String,
    multisign_address: Option<&str>,
) -> XRPLModelResult<BatchSigner> {
    match multisign_address {
        Some(address) => {
            let signer = Signer::new(address.to_string(), signature, wallet.public_key.clone());

            BatchSigner::multi(batch_account, alloc::vec![signer])
        }
        None => Ok(BatchSigner::single(
            batch_account,
            wallet.public_key.clone(),
            signature,
        )),
    }
}

pub fn combine_batch_signers<'a>(fragments: &[Batch<'a>]) -> XRPLModelResult<Batch<'a>> {
    let first = fragments
        .first()
        .ok_or_else(|| invalid("transactions", "at least one transaction", "0 transactions"))?;

    for fragment in fragments {
        validate_fragment(fragment)?;
    }

    validate_equivalence(first, &fragments[1..])?;

    let outer_account = first.common_fields.account.to_string();
    let mut combined = first.clone();

    combined.batch_signers = Some(merge_fragment_signers(fragments, &outer_account)?);

    Ok(combined)
}

/// A fragment needs `BatchSigners` and must be unsigned.
fn validate_fragment(fragment: &Batch<'_>) -> XRPLModelResult<()> {
    if fragment.batch_signers.as_ref().is_none_or(|s| s.is_empty()) {
        return Err(invalid(
            "batch_signers",
            "every fragment to include BatchSigners",
            "missing or empty",
        ));
    }

    let common = &fragment.common_fields;
    let unsigned = common.signing_pub_key.as_deref().is_none_or(str::is_empty)
        && common.txn_signature.is_none()
        && common.signers.is_none();
    if !unsigned {
        return Err(invalid("transaction", "an unsigned Batch", "signed"));
    }

    Ok(())
}

/// Everything bound into a Batch signature: outer account, sequence value,
/// flags, inner transaction IDs.
type EquivalenceKey = (String, u32, u32, Vec<String>);

/// Everything bound into a Batch signature: outer account, sequence value,
/// flags, inner transaction IDs. Fragments that differ were signed over
/// different payloads and can't be combined.
fn equivalence_key(batch: &Batch<'_>) -> XRPLModelResult<EquivalenceKey> {
    let ids = batch
        .raw_transactions
        .iter()
        .map(|v| v.transaction_hash())
        .collect::<XRPLModelResult<Vec<String>>>()?;

    Ok((
        batch.common_fields.account.to_string(),
        batch.outer_sequence()?,
        batch.mode_flags(),
        ids,
    ))
}

/// Fragments that differ in any signed field were signed over different
/// payloads and can't be combined.
fn validate_equivalence(first: &Batch<'_>, rest: &[Batch<'_>]) -> XRPLModelResult<()> {
    let key = equivalence_key(first)?;
    for fragment in rest {
        if equivalence_key(fragment)? != key {
            return Err(invalid(
                "transactions",
                "same account, sequence, flags and inner transaction IDs",
                "mismatch",
            ));
        }
    }

    Ok(())
}

/// Groups every fragment's `BatchSigners` by account, merging entries that
/// share one, and returns them in AccountID order.
fn merge_fragment_signers(
    fragments: &[Batch<'_>],
    outer_account: &str,
) -> XRPLModelResult<Vec<BatchSigner>> {
    let mut by_account: BTreeMap<String, BatchSigner> = BTreeMap::new();

    for signer in fragments
        .iter()
        .flat_map(|f| f.batch_signers.iter().flatten())
    {
        if signer.account == outer_account {
            continue; // rippled: temBAD_SIGNER
        }
        let merged = match by_account.remove(&signer.account) {
            Some(previous) => merge_batch_signers(previous, signer.clone())?,
            None => signer.clone(),
        };
        by_account.insert(signer.account.clone(), merged);
    }

    sort_by_account_id(by_account.into_values().collect(), |s| s.account.as_str())
}

/// Merge two BatchSigners for the same account: multi-signed fragments have
/// their inner signers merged and sorted; identical single-signed fragments
/// collapse; any other combination is an error.
fn merge_batch_signers(first: BatchSigner, second: BatchSigner) -> XRPLModelResult<BatchSigner> {
    match (&first.signers, &second.signers) {
        (Some(a), Some(b)) => {
            let mut by_account: BTreeMap<String, Signer> = BTreeMap::new();
            for inner in a.iter().chain(b.iter()) {
                if by_account
                    .get(&inner.account)
                    .is_some_and(|existing| existing != inner)
                {
                    return Err(invalid(
                        "batch_signers",
                        "consistent signatures per inner signer",
                        alloc::format!(
                            "conflicting signatures for {} on {}",
                            inner.account,
                            first.account
                        ),
                    ));
                }
                by_account.insert(inner.account.clone(), inner.clone());
            }

            // `multi` sorts the inner signers by AccountID.
            BatchSigner::multi(first.account, by_account.into_values().collect())
        }
        (None, None) => {
            if first.signing_pub_key == second.signing_pub_key
                && first.txn_signature == second.txn_signature
            {
                Ok(first)
            } else {
                Err(invalid(
                    "batch_signers",
                    "identical single-signed fragments per account",
                    alloc::format!("conflicting signatures for {}", first.account),
                ))
            }
        }
        _ => Err(invalid(
            "batch_signers",
            "all fragments for an account single-signed or all multi-signed",
            alloc::format!("mixed fragments for {}", first.account),
        )),
    }
}

/// Merge the `BatchSigners` of several fragments of one Batch into a single
/// Batch, ordered by AccountID, with the outer account's entries dropped.
pub fn merge_batch_signer_fragments<'a>(fragments: &[Batch<'a>]) -> XRPLModelResult<Batch<'a>> {
    let first = fragments
        .first()
        .ok_or_else(|| invalid("transactions", "at least one transaction", "0 transactions"))?;

    for fragment in fragments {
        if fragment.batch_signers.as_ref().is_none_or(|s| s.is_empty()) {
            return Err(invalid(
                "batch_signers",
                "every fragment to include BatchSigners",
                "missing or empty",
            ));
        }
        let c = &fragment.common_fields;
        let unsigned = c.signing_pub_key.as_deref().is_none_or(str::is_empty)
            && c.txn_signature.is_none()
            && c.signers.is_none();
        if !unsigned {
            return Err(invalid("transaction", "an unsigned Batch", "signed"));
        }
    }

    let key = equivalence_key(first)?;
    for fragment in &fragments[1..] {
        if equivalence_key(fragment)? != key {
            return Err(invalid(
                "transactions",
                "same account, sequence, flags and inner transaction IDs",
                "mismatch",
            ));
        }
    }

    let outer = first.common_fields.account.to_string();
    let mut by_account: BTreeMap<String, BatchSigner> = BTreeMap::new();
    for fragment in fragments {
        for signer in fragment.batch_signers.iter().flatten() {
            if signer.account == outer {
                continue; // rippled: temBAD_SIGNER
            }
            let merged = match by_account.remove(&signer.account) {
                Some(previous) => merge_batch_signers(previous, signer.clone())?,
                None => signer.clone(),
            };
            by_account.insert(signer.account.clone(), merged);
        }
    }

    let sorted = sort_by_account_id(by_account.into_values().collect(), |s| s.account.as_str())?;
    let mut combined = first.clone();

    combined.batch_signers = Some(sorted);

    Ok(combined)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::models::transactions::batch::{BatchFlag, RawTransaction};
    use crate::models::transactions::payment::Payment;
    use crate::models::transactions::{CommonFields, Signer, TransactionType};
    use crate::models::Amount;

    const OUTER: &str = "rU4EE1FskCPJw5QkLx1iGgdWiJa6HeqYyb";
    const DEST: &str = "rLSn6Z3T8uCxbcd1oxwfGQN1Fdn5CyGujK";
    const OTHER_A: &str = "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn";
    const OTHER_B: &str = "ra5nK24KXen9AHvsdFTKHSANinZseWnPcX";

    fn payment_from(account: &str, sequence: u32) -> RawTransaction {
        let payment = Payment {
            common_fields: CommonFields {
                account: account.into(),
                transaction_type: TransactionType::Payment,
                sequence: Some(sequence),
                ..Default::default()
            },
            amount: Amount::XRPAmount("1000000".into()),
            destination: DEST.into(),
            ..Default::default()
        };

        RawTransaction::from_transaction(&payment).unwrap()
    }

    fn payment(sequence: u32) -> RawTransaction {
        payment_from(OUTER, sequence)
    }

    fn signer(account: &str) -> Signer {
        Signer::new(account.to_string(), "BB".to_string(), "02AA".to_string())
    }

    fn batch() -> Batch<'static> {
        Batch::new(
            OUTER,
            BatchFlag::TfAllOrNothing,
            vec![payment(4), payment(5)],
            None,
        )
        .with_sequence(3)
        .with_fee("40")
    }

    fn base() -> Batch<'static> {
        Batch::new(
            OUTER,
            BatchFlag::TfAllOrNothing,
            vec![
                payment_from(OUTER, 4),
                payment_from(OTHER_A, 1),
                payment_from(OTHER_B, 1),
            ],
            None,
        )
        .with_sequence(3)
        .with_fee("40")
    }

    fn with(signer: BatchSigner) -> Batch<'static> {
        base().with_batch_signers(vec![signer]).unwrap()
    }

    fn single(account: &str, sig: &str) -> BatchSigner {
        BatchSigner::single(account, "02AA", sig)
    }

    #[test]
    fn combine_merges_and_sorts() {
        let merged = merge_batch_signer_fragments(&[
            with(single(OTHER_B, "BB")),
            with(single(OTHER_A, "AA")),
            with(single(OTHER_A, "AA")), // identical duplicate collapses
        ])
        .unwrap();
        let signers = merged.batch_signers.unwrap();
        assert_eq!(signers.len(), 2);
        assert!(
            combine_batch_signers(&[with(single(OTHER_A, "AA")), with(single(OTHER_B, "BB"))])
                .is_ok()
        );
    }

    #[test]
    fn combine_rejects_bad_input() {
        assert!(merge_batch_signer_fragments(&[]).is_err());
        assert!(
            merge_batch_signer_fragments(&[base()]).is_err(),
            "no BatchSigners"
        );

        let mut signed = with(single(OTHER_A, "AA"));
        signed.common_fields.txn_signature = Some("AB".into());
        assert!(
            merge_batch_signer_fragments(&[signed]).is_err(),
            "not unsigned"
        );

        let other_seq = with(single(OTHER_B, "BB")).with_sequence(9);
        assert!(merge_batch_signer_fragments(&[with(single(OTHER_A, "AA")), other_seq]).is_err());

        assert!(
            merge_batch_signer_fragments(&[
                with(single(OTHER_A, "AA")),
                with(single(OTHER_A, "XX")),
            ])
            .is_err(),
            "conflicting single signatures"
        );
    }

    #[test]
    fn merge_multisigned_fragments() {
        let s = |acct: &str, sig: &str| {
            Signer::new(acct.to_string(), sig.to_string(), "02AA".to_string())
        };
        let f1 = with(BatchSigner::multi(OTHER_A, vec![s(OTHER_B, "11")]).unwrap());
        let f2 = with(BatchSigner::multi(OTHER_A, vec![s(DEST, "22")]).unwrap());
        let merged = merge_batch_signer_fragments(&[f1.clone(), f2]).unwrap();
        assert_eq!(
            merged.batch_signers.unwrap()[0]
                .signers
                .as_ref()
                .unwrap()
                .len(),
            2
        );

        let conflict = with(BatchSigner::multi(OTHER_A, vec![s(OTHER_B, "99")]).unwrap());
        assert!(merge_batch_signer_fragments(&[f1.clone(), conflict]).is_err());
        assert!(
            merge_batch_signer_fragments(&[f1, with(single(OTHER_A, "AA"))]).is_err(),
            "mixed"
        );
    }

    #[test]
    fn outer_account_signer_is_dropped_before_validation() {
        // An outer-account entry in a fragment is skipped, not an error.
        let mut frag = with(single(OTHER_A, "AA"));
        frag.batch_signers
            .as_mut()
            .unwrap()
            .push(single(OUTER, "ZZ"));
        let merged = merge_batch_signer_fragments(&[frag]).unwrap();
        assert_eq!(merged.batch_signers.unwrap().len(), 1);
    }

    #[test]
    fn sign_checks_and_fragments() {
        let wallet = Wallet::create(None).unwrap();
        // Wallet address isn't in the batch.
        assert!(sign_multi_batch(&wallet, &base(), MultisignMode::Off, None).is_err());
        // Signer equals the batch account.
        assert!(sign_multi_batch(
            &wallet,
            &base(),
            MultisignMode::Address(OTHER_A),
            Some(OTHER_A)
        )
        .is_err());

        let signed = sign_multi_batch(&wallet, &base(), MultisignMode::Off, Some(OTHER_A)).unwrap();

        assert_eq!(signed.batch_signers.as_ref().unwrap().len(), 1);

        let multi = sign_multi_batch(
            &wallet,
            &base(),
            MultisignMode::WalletAddress,
            Some(OTHER_B),
        )
        .unwrap();
        assert!(multi.batch_signers.unwrap()[0].signers.is_some());
    }
}
