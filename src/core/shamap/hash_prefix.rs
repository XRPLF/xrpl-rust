//! Hash prefix constants used in ShaMap hashing.
//!
//! Each prefix is a 4-byte big-endian value prepended to data before hashing,
//! ensuring domain separation between different hash contexts in the XRP Ledger.

/// Transaction ID hash prefix (0x54584E00 = "TXN\0")
pub const TRANSACTION_ID: [u8; 4] = [0x54, 0x58, 0x4E, 0x00];

/// Transaction node (tx + metadata) hash prefix for ShaMap leaves (0x534E4400 = "SND\0")
pub const TRANSACTION: [u8; 4] = [0x53, 0x4E, 0x44, 0x00];

/// Account state entry hash prefix (0x4D4C4E00 = "MLN\0")
pub const ACCOUNT_STATE_ENTRY: [u8; 4] = [0x4D, 0x4C, 0x4E, 0x00];

/// Inner node hash prefix (0x4D494E00 = "MIN\0")
pub const INNER_NODE: [u8; 4] = [0x4D, 0x49, 0x4E, 0x00];

/// Ledger header hash prefix (0x4C575200 = "LWR\0")
pub const LEDGER_HEADER: [u8; 4] = [0x4C, 0x57, 0x52, 0x00];

/// Transaction signing hash prefix (0x53545800 = "STX\0")
pub const TRANSACTION_SIG: [u8; 4] = [0x53, 0x54, 0x58, 0x00];

/// Transaction multi-signing hash prefix (0x534D5400 = "SMT\0")
pub const TRANSACTION_MULTI_SIG: [u8; 4] = [0x53, 0x4D, 0x54, 0x00];

/// Validation hash prefix (0x56414C00 = "VAL\0")
pub const VALIDATION: [u8; 4] = [0x56, 0x41, 0x4C, 0x00];

/// Proposal hash prefix (0x50525000 = "PRP\0")
pub const PROPOSAL: [u8; 4] = [0x50, 0x52, 0x50, 0x00];

/// Payment channel claim hash prefix (0x434C4D00 = "CLM\0")
pub const PAYMENT_CHANNEL_CLAIM: [u8; 4] = [0x43, 0x4C, 0x4D, 0x00];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_id_prefix() {
        assert_eq!(
            u32::from_be_bytes(TRANSACTION_ID),
            0x54584E00,
            "TRANSACTION_ID must be 0x54584E00"
        );
    }

    #[test]
    fn test_transaction_prefix() {
        assert_eq!(
            u32::from_be_bytes(TRANSACTION),
            0x534E4400,
            "TRANSACTION must be 0x534E4400"
        );
    }

    #[test]
    fn test_account_state_entry_prefix() {
        assert_eq!(
            u32::from_be_bytes(ACCOUNT_STATE_ENTRY),
            0x4D4C4E00,
            "ACCOUNT_STATE_ENTRY must be 0x4D4C4E00"
        );
    }

    #[test]
    fn test_inner_node_prefix() {
        assert_eq!(
            u32::from_be_bytes(INNER_NODE),
            0x4D494E00,
            "INNER_NODE must be 0x4D494E00"
        );
    }

    #[test]
    fn test_ledger_header_prefix() {
        assert_eq!(
            u32::from_be_bytes(LEDGER_HEADER),
            0x4C575200,
            "LEDGER_HEADER must be 0x4C575200"
        );
    }

    #[test]
    fn test_transaction_sig_prefix() {
        assert_eq!(
            u32::from_be_bytes(TRANSACTION_SIG),
            0x53545800,
            "TRANSACTION_SIG must be 0x53545800"
        );
    }

    #[test]
    fn test_transaction_multi_sig_prefix() {
        assert_eq!(
            u32::from_be_bytes(TRANSACTION_MULTI_SIG),
            0x534D5400,
            "TRANSACTION_MULTI_SIG must be 0x534D5400"
        );
    }

    #[test]
    fn test_validation_prefix() {
        assert_eq!(
            u32::from_be_bytes(VALIDATION),
            0x56414C00,
            "VALIDATION must be 0x56414C00"
        );
    }

    #[test]
    fn test_proposal_prefix() {
        assert_eq!(
            u32::from_be_bytes(PROPOSAL),
            0x50525000,
            "PROPOSAL must be 0x50525000"
        );
    }

    #[test]
    fn test_payment_channel_claim_prefix() {
        assert_eq!(
            u32::from_be_bytes(PAYMENT_CHANNEL_CLAIM),
            0x434C4D00,
            "PAYMENT_CHANNEL_CLAIM must be 0x434C4D00"
        );
    }
}
