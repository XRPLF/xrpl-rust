//! Ledger hash computation for XRP Ledger headers and state trees.
//!
//! Implements the canonical hash computation for ledger headers, transaction
//! trees, and account state trees as defined in the XRP Ledger protocol.

use alloc::vec::Vec;

use super::hash_prefix;
use super::sha512half::Sha512Half;
use super::tree::{Hash256, ShaMap, ZERO_256};

/// An XRP Ledger header containing all fields needed for hash computation.
pub struct LedgerHeader {
    /// The sequence number of this ledger (u32).
    pub ledger_index: u32,
    /// Total drops of XRP in existence (u64).
    pub total_coins: u64,
    /// Hash of the previous ledger header.
    pub parent_hash: Hash256,
    /// Root hash of the transaction ShaMap.
    pub transaction_hash: Hash256,
    /// Root hash of the account state ShaMap.
    pub account_hash: Hash256,
    /// Close time of the parent ledger (seconds since Ripple epoch).
    pub parent_close_time: u32,
    /// Close time of this ledger (seconds since Ripple epoch).
    pub close_time: u32,
    /// Close time resolution in seconds (typically 10).
    pub close_time_resolution: u8,
    /// Close flags (bit field).
    pub close_flags: u8,
}

/// Compute the canonical hash of a ledger header.
///
/// Serialization order (all integers big-endian):
/// `LEDGER_HEADER_PREFIX || ledger_index(4B) || total_coins(8B) ||
///  parent_hash(32B) || transaction_hash(32B) || account_hash(32B) ||
///  parent_close_time(4B) || close_time(4B) || resolution(1B) || flags(1B)`
pub fn ledger_hash(header: &LedgerHeader) -> Hash256 {
    let mut h = Sha512Half::new();

    h.update(&hash_prefix::LEDGER_HEADER);
    h.update(&header.ledger_index.to_be_bytes());
    h.update(&header.total_coins.to_be_bytes());
    h.update(&header.parent_hash);
    h.update(&header.transaction_hash);
    h.update(&header.account_hash);
    h.update(&header.parent_close_time.to_be_bytes());
    h.update(&header.close_time.to_be_bytes());
    h.update(&[header.close_time_resolution]);
    h.update(&[header.close_flags]);

    h.finish()
}

/// A transaction item for building the transaction ShaMap.
pub struct TransactionItem {
    /// The 256-bit transaction hash used as the ShaMap index.
    pub index: Hash256,
    /// Serialized transaction data (transaction + metadata blob).
    pub data: Vec<u8>,
}

/// Compute the root hash of a transaction tree from a list of transaction items.
///
/// Each transaction is inserted into a ShaMap with the `TRANSACTION` hash prefix
/// (which covers tx + metadata for leaf nodes).
pub fn transaction_tree_hash(transactions: &[TransactionItem]) -> Hash256 {
    if transactions.is_empty() {
        return ZERO_256;
    }

    let mut map = ShaMap::new();
    for tx in transactions {
        map.add_item(tx.index, hash_prefix::TRANSACTION, tx.data.clone());
    }
    map.hash()
}

/// An account state entry for building the account state ShaMap.
pub struct AccountStateItem {
    /// The 256-bit key (ledger entry index) used as the ShaMap index.
    pub index: Hash256,
    /// Serialized account state data.
    pub data: Vec<u8>,
}

/// Compute the root hash of an account state tree from a list of state entries.
///
/// Each entry is inserted into a ShaMap with the `ACCOUNT_STATE_ENTRY` hash prefix.
pub fn account_state_hash(entries: &[AccountStateItem]) -> Hash256 {
    if entries.is_empty() {
        return ZERO_256;
    }

    let mut map = ShaMap::new();
    for entry in entries {
        map.add_item(
            entry.index,
            hash_prefix::ACCOUNT_STATE_ENTRY,
            entry.data.clone(),
        );
    }
    map.hash()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_ledger_hash_deterministic() {
        let header = LedgerHeader {
            ledger_index: 1000,
            total_coins: 99_999_999_999_000_000,
            parent_hash: [0xAA; 32],
            transaction_hash: [0xBB; 32],
            account_hash: [0xCC; 32],
            parent_close_time: 700_000_000,
            close_time: 700_000_010,
            close_time_resolution: 10,
            close_flags: 0,
        };

        let hash1 = ledger_hash(&header);
        let hash2 = ledger_hash(&header);
        assert_eq!(
            hash1, hash2,
            "identical headers must produce identical hashes"
        );
        assert_ne!(hash1, ZERO_256, "ledger hash should not be zero");
    }

    #[test]
    fn test_ledger_hash_serialization() {
        let header = LedgerHeader {
            ledger_index: 1,
            total_coins: 100_000_000_000,
            parent_hash: ZERO_256,
            transaction_hash: ZERO_256,
            account_hash: ZERO_256,
            parent_close_time: 0,
            close_time: 0,
            close_time_resolution: 10,
            close_flags: 0,
        };

        // Manually build the expected input
        let mut expected_input = Vec::new();
        expected_input.extend_from_slice(&hash_prefix::LEDGER_HEADER);
        expected_input.extend_from_slice(&1u32.to_be_bytes());
        expected_input.extend_from_slice(&100_000_000_000u64.to_be_bytes());
        expected_input.extend_from_slice(&ZERO_256);
        expected_input.extend_from_slice(&ZERO_256);
        expected_input.extend_from_slice(&ZERO_256);
        expected_input.extend_from_slice(&0u32.to_be_bytes());
        expected_input.extend_from_slice(&0u32.to_be_bytes());
        expected_input.push(10);
        expected_input.push(0);

        let expected_hash = super::super::sha512half::sha512half(&expected_input);
        assert_eq!(ledger_hash(&header), expected_hash);
    }

    #[test]
    fn test_ledger_hash_changes_with_index() {
        let h1 = ledger_hash(&LedgerHeader {
            ledger_index: 1,
            total_coins: 0,
            parent_hash: ZERO_256,
            transaction_hash: ZERO_256,
            account_hash: ZERO_256,
            parent_close_time: 0,
            close_time: 0,
            close_time_resolution: 10,
            close_flags: 0,
        });

        let h2 = ledger_hash(&LedgerHeader {
            ledger_index: 2,
            total_coins: 0,
            parent_hash: ZERO_256,
            transaction_hash: ZERO_256,
            account_hash: ZERO_256,
            parent_close_time: 0,
            close_time: 0,
            close_time_resolution: 10,
            close_flags: 0,
        });

        assert_ne!(
            h1, h2,
            "different ledger indices must produce different hashes"
        );
    }

    #[test]
    fn test_empty_transaction_tree() {
        assert_eq!(
            transaction_tree_hash(&[]),
            ZERO_256,
            "empty transaction tree must hash to zero"
        );
    }

    #[test]
    fn test_single_transaction_tree() {
        let tx = TransactionItem {
            index: [0xAA; 32],
            data: vec![1, 2, 3, 4],
        };
        let hash = transaction_tree_hash(&[tx]);
        assert_ne!(hash, ZERO_256);
    }

    #[test]
    fn test_transaction_tree_order_independence() {
        let tx1 = TransactionItem {
            index: [0x11; 32],
            data: vec![1, 2, 3],
        };
        let tx2 = TransactionItem {
            index: [0x22; 32],
            data: vec![4, 5, 6],
        };

        let hash_a = transaction_tree_hash(&[
            TransactionItem {
                index: tx1.index,
                data: tx1.data.clone(),
            },
            TransactionItem {
                index: tx2.index,
                data: tx2.data.clone(),
            },
        ]);

        let hash_b = transaction_tree_hash(&[
            TransactionItem {
                index: tx2.index,
                data: tx2.data.clone(),
            },
            TransactionItem {
                index: tx1.index,
                data: tx1.data.clone(),
            },
        ]);

        assert_eq!(
            hash_a, hash_b,
            "transaction tree hash must be order-independent"
        );
    }

    #[test]
    fn test_empty_account_state() {
        assert_eq!(
            account_state_hash(&[]),
            ZERO_256,
            "empty account state must hash to zero"
        );
    }

    #[test]
    fn test_single_account_state() {
        let entry = AccountStateItem {
            index: [0xCC; 32],
            data: vec![10, 20, 30],
        };
        let hash = account_state_hash(&[entry]);
        assert_ne!(hash, ZERO_256);
    }

    #[test]
    fn test_account_state_uses_correct_prefix() {
        // Inserting the same data with different tree functions should produce
        // different hashes because they use different hash prefixes.
        let index = [0xDD; 32];
        let data = vec![1, 2, 3, 4, 5];

        let tx_hash = transaction_tree_hash(&[TransactionItem {
            index,
            data: data.clone(),
        }]);

        let acct_hash = account_state_hash(&[AccountStateItem {
            index,
            data: data.clone(),
        }]);

        assert_ne!(
            tx_hash, acct_hash,
            "different prefixes must produce different hashes"
        );
    }

    // -------------------------------------------------------------------
    // xrpl.js ledger hash test vector — ported from:
    //   packages/xrpl/test/fixtures/requests/hashLedger.json
    //   packages/xrpl/test/hashLedger.test.ts
    // -------------------------------------------------------------------

    // -------------------------------------------------------------------
    // Protocol vector tests for the index-appended leaf paths
    // (tnTRANSACTION_MD and tnACCOUNT_STATE).
    //
    // Source: packages/ripple-binary-codec/test/fixtures/ledger-full-38129.json
    // Vectors computed with ripple-binary-codec v3 (confirmed against the
    // official ledger fixture where account_hash and transaction_hash are
    // published ground truth from a real XRP Ledger close).
    // -------------------------------------------------------------------

    /// Known-answer test for `transaction_tree_hash` using a real Payment
    /// transaction from XRP Ledger 38129.
    ///
    /// The `data` blob is `writeLengthEncoded(STObject(tx)) ||
    /// writeLengthEncoded(STObject(meta))` as produced by ripple-binary-codec's
    /// `transactionItemizer`. The expected root hash matches the
    /// `transaction_hash` field published in the ledger fixture.
    ///
    /// Source: packages/ripple-binary-codec/test/fixtures/ledger-full-38129.json
    #[test]
    fn test_xrpljs_transaction_tree_hash_vector() {
        // Transaction hash (ShaMap index) from ledger 38129
        let index = hex_to_32("3B1A4E1C9BB6A7208EB146BCDB86ECEA6068ED01466D933528CA2B4C64F753EF");

        // Serialized blob: writeLengthEncoded(tx_stobject) || writeLengthEncoded(meta_stobject)
        // Produced by ripple-binary-codec transactionItemizer for the single
        // Payment transaction in ledger-full-38129.json.
        let data = hex::decode(concat!(
            "B71200002200000000240000003E6140000002540BE400684000000000000",
            "00A7321034AADB09CFF4A4804073701EC53C3510CDC95917C2BB0150FB74",
            "2D0C66E6CEE9E74473045022022EB32AECEF7C644C891C19F87966DF9C62",
            "B1F34BABA6BE774325E4BB8E2DD62022100A51437898C28C2B297112DF813",
            "1F2BB39EA5FE613487DDD611525F17962646398114550FC62003E785DC231A",
            "1058A05E56E3F09CF4E68314D4CC8AB5B21D86A82C3E9E8D0ECF2404B77FE",
            "CBAC122201C00000000F8E3110061564C6ACBD635B0F07101F7FA25871B09",
            "25F8836155462152172755845CE691C49EE824000000016240000002540BE4",
            "008114D4CC8AB5B21D86A82C3E9E8D0ECF2404B77FECBAE1E1E51100612500",
            "007A55552485FDC606352F1B0785DA5DE96FB9DBAF43EB60ECBB01B7F6FA97",
            "0F512CDA5F56B33FDD5CF3445E1A7F2BE9B06336BEBD73A5E3EE885D3EF93F",
            "7E3E2992E46F1AE6240000003E62400000E6D8EEB01EE1E722000000002400",
            "00003F2D0000000062400000E484E2CC148114550FC62003E785DC231A1058",
            "A05E56E3F09CF4E6E1E1F1031000"
        ))
        .unwrap();

        // Expected: the `transaction_hash` field from ledger-full-38129.json
        let expected =
            hex_to_32("DB83BF807416C5B3499A73130F843CF615AB8E797D79FE7D330ADF1BFA93951A");

        let result = transaction_tree_hash(&[TransactionItem { index, data }]);

        assert_eq!(
            result, expected,
            "transaction_tree_hash must match xrpl.js ledger-38129 vector"
        );
    }

    /// Known-answer test for `account_state_hash` using two real AccountRoot
    /// entries from XRP Ledger 38129.
    ///
    /// The `data` for each entry is `serializeObject(entry)` (without the
    /// `index` field) as produced by ripple-binary-codec's `entryItemizer`.
    /// The expected root hash was computed by ripple-binary-codec
    /// `accountStateHash([entry1, entry2])` and verified to be
    /// order-independent.
    ///
    /// Source: packages/ripple-binary-codec/test/fixtures/ledger-full-38129.json
    #[test]
    fn test_xrpljs_account_state_hash_vector() {
        // Entry 1: AccountRoot for rBKPS4oLSaV2KVVuHH8EpQqMGgGefGFQs7
        let idx1 = hex_to_32("02CE52E3E46AD340B1C7900F86AFB959AE0C246916E3463905EDD61DE26FFFDD");
        let data1 = hex::decode(concat!(
            "1100612200000000240000000125000022C52D00000000558D7F42ED0621FB",
            "CFAE55CC6F2A9403A2AFB205708CCBA3109BB61DB8DDA261B46240000000160",
            "DC0808114712B799C79D1EEE3094B59EF9920C7FEB3CE4499"
        ))
        .unwrap();

        // Entry 2: AccountRoot for rLs1MzkFWCxTbuAHgjeTZK4fcCDDnf2KRv
        let idx2 = hex_to_32("032D4205B5D7DCEC8A4E56851C44555F6DC7D410AA823AE140C78674B8734DBF");
        let data2 = hex::decode(concat!(
            "1100612200000000240000000125000000072D0000000055DF530FB14C530485",
            "2F20080B0A8EEF3A6BDD044F41F4EBBD68B8B321145FE4FF6240000002540BE",
            "4008114D0F5430B66E06498D4CEEC816C7B3337F9982337"
        ))
        .unwrap();

        // Expected: ripple-binary-codec accountStateHash([entry1, entry2])
        // Verified order-independent by the generating script.
        let expected =
            hex_to_32("9CCA89E983211E80529E590EE9AE9646A5292380A05623D398D206FCAE81DE35");

        let result = account_state_hash(&[
            AccountStateItem {
                index: idx1,
                data: data1.clone(),
            },
            AccountStateItem {
                index: idx2,
                data: data2.clone(),
            },
        ]);

        assert_eq!(
            result, expected,
            "account_state_hash must match xrpl.js ledger-38129 vector"
        );

        // Also verify order independence
        let result_rev = account_state_hash(&[
            AccountStateItem {
                index: idx2,
                data: data2,
            },
            AccountStateItem {
                index: idx1,
                data: data1,
            },
        ]);

        assert_eq!(
            result, result_rev,
            "account_state_hash must be order-independent"
        );
    }

    /// Helper: decode a hex string to a 32-byte array.
    fn hex_to_32(hex: &str) -> [u8; 32] {
        let bytes = hex::decode(hex).unwrap();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        arr
    }

    /// xrpl.js hashLedger test vector: real ledger header with known hash.
    ///
    /// Source: packages/xrpl/test/fixtures/requests/hashLedger.json
    /// Expected: F4D865D83EB88C1A1911B9E90641919A1314F36E1B099F8E95FE3B7C77BE3349
    #[test]
    fn test_xrpljs_ledger_hash_vector() {
        let header = LedgerHeader {
            ledger_index: 15_202_439,
            total_coins: 99_998_831_688_050_493,
            parent_hash: hex_to_32(
                "12724A65B030C15A1573AA28B1BBB5DF3DA4589AA3623675A31CAE69B23B1C4E",
            ),
            transaction_hash: hex_to_32(
                "325EACC5271322539EEEC2D6A5292471EF1B3E72AE7180533EFC3B8F0AD435C8",
            ),
            account_hash: hex_to_32(
                "D9ABF622DA26EEEE48203085D4BC23B0F77DC6F8724AC33D975DA3CA492D2E44",
            ),
            parent_close_time: 492_656_460,
            close_time: 492_656_470,
            close_time_resolution: 10,
            close_flags: 0,
        };

        let expected =
            hex_to_32("F4D865D83EB88C1A1911B9E90641919A1314F36E1B099F8E95FE3B7C77BE3349");
        let computed = ledger_hash(&header);

        assert_eq!(
            computed, expected,
            "ledger hash must match xrpl.js test vector"
        );
    }
}
