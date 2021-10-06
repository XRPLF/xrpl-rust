//! Maps and helpers providing serialization-related
//! information about fields.

use crate::core::binarycodec::definitions::exceptions::XRPDefinitionException;
use crate::core::binarycodec::definitions::field_info::FieldInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Types {
    pub validation: u16,
    pub done: i16,
    pub hash_128: u16,
    pub blob: u16,
    #[serde(rename = "AccountID")]
    pub account_id: u16,
    pub amount: u16,
    pub hash_256: u16,
    pub u_int_8: u16,
    pub vector_256: u16,
    pub serialized_dict: u16,
    pub unknown: i16,
    pub transaction: u16,
    pub hash_160: u16,
    pub path_set: u16,
    pub ledger_entry: u16,
    pub u_int_16: u16,
    pub not_present: u16,
    pub u_int_64: u16,
    pub u_int_32: u16,
    pub serialized_list: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LedgerEntryTypes {
    pub any: i16,
    pub child: i16,
    pub invalid: i16,
    pub account_root: u16,
    pub directory_node: u16,
    pub ripple_state: u16,
    pub ticket: u16,
    pub signer_list: u16,
    pub offer: u16,
    pub ledger_hashes: u16,
    pub amendments: u16,
    pub fee_settings: u16,
    pub escrow: u16,
    pub pay_channel: u16,
    pub deposit_preauth: u16,
    pub check: u16,
    pub nickname: u16,
    pub contract: u16,
    pub generator_map: u16,
    #[serde(rename = "NegativeUNL")]
    pub negative_unl: u16,
}

/// =(
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResults {
    #[serde(rename = "telLOCAL_ERROR")]
    pub tel_local_error: i16,
    #[serde(rename = "telBAD_DOMAIN")]
    pub tel_bad_domain: i16,
    #[serde(rename = "telBAD_PATH_COUNT")]
    pub tel_bad_path_count: i16,
    #[serde(rename = "telBAD_PUBLIC_KEY")]
    pub tel_bad_public_keyt: i16,
    #[serde(rename = "telFAILED_PROCESSING")]
    pub tel_failed_processing: i16,
    #[serde(rename = "telINSUF_FEE_P")]
    pub tel_insuf_fee_p: i16,
    #[serde(rename = "telNO_DST_PARTIAL")]
    pub tel_no_dst_partial: i16,
    #[serde(rename = "telCAN_NOT_QUEUE")]
    pub tel_can_not_queue: i16,
    #[serde(rename = "telCAN_NOT_QUEUE_BALANCE")]
    pub tel_can_not_queue_balance: i16,
    #[serde(rename = "telCAN_NOT_QUEUE_BLOCKS")]
    pub tel_can_not_blocks: i16,
    #[serde(rename = "telCAN_NOT_QUEUE_BLOCKED")]
    pub tel_can_not_blocked: i16,
    #[serde(rename = "telCAN_NOT_QUEUE_FEE")]
    pub tel_can_not_queue_fee: i16,
    #[serde(rename = "telCAN_NOT_QUEUE_FULL")]
    pub tel_can_not_queue_null: i16,

    #[serde(rename = "temMALFORMED")]
    pub tem_malformed: i16,
    #[serde(rename = "temBAD_AMOUNT")]
    pub tem_bad_amount: i16,
    #[serde(rename = "temBAD_CURRENCY")]
    pub tem_bad_currency: i16,
    #[serde(rename = "temBAD_EXPIRATION")]
    pub tem_bad_expiration: i16,
    #[serde(rename = "temBAD_FEE")]
    pub tem_bad_fee: i16,
    #[serde(rename = "temBAD_ISSUER")]
    pub tem_bad_issuer: i16,
    #[serde(rename = "temBAD_LIMIT")]
    pub tem_bad_limit: i16,
    #[serde(rename = "temBAD_OFFER")]
    pub tem_bad_offer: i16,
    #[serde(rename = "temBAD_PATH")]
    pub tem_bad_path: i16,
    #[serde(rename = "temBAD_PATH_LOOP")]
    pub tem_bad_path_loop: i16,
    #[serde(rename = "temBAD_REGKEY")]
    pub tem_bad_regkey: i16,
    #[serde(rename = "temBAD_SEND_XRP_LIMIT")]
    pub tem_bad_send_xrp_limit: i16,
    #[serde(rename = "temBAD_SEND_XRP_MAX")]
    pub tem_bad_send_xrp_max: i16,
    #[serde(rename = "temBAD_SEND_XRP_NO_DIRECT")]
    pub tem_bad_send_xrp_no_direct: i16,
    #[serde(rename = "temBAD_SEND_XRP_PARTIAL")]
    pub tem_bad_send_xrp_partial: i16,
    #[serde(rename = "temBAD_SEND_XRP_PATHS")]
    pub tem_bad_send_xrp_paths: i16,
    #[serde(rename = "temBAD_SEQUENCE")]
    pub tem_bad_sequence: i16,
    #[serde(rename = "temBAD_SIGNATURE")]
    pub tem_bad_signature: i16,
    #[serde(rename = "temBAD_SRC_ACCOUNT")]
    pub tem_bad_src_account: i16,
    #[serde(rename = "temBAD_TRANSFER_RATE")]
    pub tem_bad_transfer_rate: i16,
    #[serde(rename = "temDST_IS_SRC")]
    pub tem_dst_is_src: i16,
    #[serde(rename = "temDST_NEEDED")]
    pub tem_dst_needed: i16,
    #[serde(rename = "temINVALID")]
    pub tem_invalid: i16,
    #[serde(rename = "temINVALID_FLAG")]
    pub tem_invalid_flag: i16,
    #[serde(rename = "temREDUNDANT")]
    pub tem_redundant: i16,
    #[serde(rename = "temRIPPLE_EMPTY")]
    pub tem_ripple_empty: i16,
    #[serde(rename = "temDISABLED")]
    pub tem_disabled: i16,
    #[serde(rename = "temBAD_SIGNER")]
    pub tem_bad_signer: i16,
    #[serde(rename = "temBAD_QUORUM")]
    pub tem_bad_quorum: i16,
    #[serde(rename = "temBAD_WEIGHT")]
    pub tem_bad_weight: i16,
    #[serde(rename = "temBAD_TICK_SIZE")]
    pub tem_bad_tick_size: i16,
    #[serde(rename = "temINVALID_ACCOUNT_ID")]
    pub tem_invalid_account_id: i16,
    #[serde(rename = "temCANNOT_PREAUTH_SELF")]
    pub tem_cannot_preauth_self: i16,
    #[serde(rename = "temUNCERTAIN")]
    pub tem_uncertain: i16,
    #[serde(rename = "temUNKNOWN")]
    pub tem_unknown: i16,

    #[serde(rename = "tefFAILURE")]
    pub tef_failure: i16,
    #[serde(rename = "tefALREADY")]
    pub tef_already: i16,
    #[serde(rename = "tefBAD_ADD_AUTH")]
    pub tef_bad_add_auth: i16,
    #[serde(rename = "tefBAD_AUTH")]
    pub tef_bad_auth: i16,
    #[serde(rename = "tefBAD_LEDGER")]
    pub tef_bad_ledger: i16,
    #[serde(rename = "tefCREATED")]
    pub tef_created: i16,
    #[serde(rename = "tefEXCEPTION")]
    pub tef_exception: i16,
    #[serde(rename = "tefINTERNAL")]
    pub tef_internal: i16,
    #[serde(rename = "tefNO_AUTH_REQUIRED")]
    pub tef_no_auth_required: i16,
    #[serde(rename = "tefPAST_SEQ")]
    pub tef_past_seq: i16,
    #[serde(rename = "tefWRONG_PRIOR")]
    pub tef_wrong_prior: i16,
    #[serde(rename = "tefMASTER_DISABLED")]
    pub tef_master_disabled: i16,
    #[serde(rename = "tefMAX_LEDGER")]
    pub tef_max_ledger: i16,
    #[serde(rename = "tefBAD_SIGNATURE")]
    pub tef_bad_signature: i16,
    #[serde(rename = "tefBAD_QUORUM")]
    pub tef_bad_quorum: i16,
    #[serde(rename = "tefNOT_MULTI_SIGNING")]
    pub tef_not_multi_signing: i16,
    #[serde(rename = "tefBAD_AUTH_MASTER")]
    pub tef_bad_auth_master: i16,
    #[serde(rename = "tefINVARIANT_FAILED")]
    pub tef_invariant_failed: i16,
    #[serde(rename = "tefTOO_BIG")]
    pub tef_too_big: i16,

    #[serde(rename = "terRETRY")]
    pub ter_retry: i16,
    #[serde(rename = "terFUNDS_SPENT")]
    pub ter_funds_spent: i16,
    #[serde(rename = "terINSUF_FEE_B")]
    pub ter_insuf_fee_b: i16,
    #[serde(rename = "terNO_ACCOUNT")]
    pub ter_no_account: i16,
    #[serde(rename = "terNO_AUTH")]
    pub ter_no_auth: i16,
    #[serde(rename = "terNO_LINE")]
    pub ter_no_line: i16,
    #[serde(rename = "terOWNERS")]
    pub ter_owners: i16,
    #[serde(rename = "terPRE_SEQ")]
    pub ter_pre_seq: i16,
    #[serde(rename = "terLAST")]
    pub ter_last: i16,
    #[serde(rename = "terNO_RIPPLE")]
    pub ter_no_ripple: i16,
    #[serde(rename = "terQUEUED")]
    pub ter_queued: i16,

    #[serde(rename = "tesSUCCESS")]
    pub tes_success: u16,

    #[serde(rename = "tecCLAIM")]
    pub tec_claim: u16,
    #[serde(rename = "tecPATH_PARTIAL")]
    pub tec_path_partial: u16,
    #[serde(rename = "tecUNFUNDED_ADD")]
    pub tec_unfunded_add: u16,
    #[serde(rename = "tecUNFUNDED_OFFER")]
    pub tec_unfunded_offer: u16,
    #[serde(rename = "tecUNFUNDED_PAYMENT")]
    pub tec_unfunded_payment: u16,
    #[serde(rename = "tecFAILED_PROCESSING")]
    pub tec_failed_processing: u16,
    #[serde(rename = "tecDIR_FULL")]
    pub tec_dir_full: u16,
    #[serde(rename = "tecINSUF_RESERVE_LINE")]
    pub tec_insuf_reserve_line: u16,
    #[serde(rename = "tecINSUF_RESERVE_OFFER")]
    pub tec_insuf_reserve_offer: u16,
    #[serde(rename = "tecNO_DST")]
    pub tec_no_dst: u16,
    #[serde(rename = "tecNO_DST_INSUF_XRP")]
    pub tec_no_dst_insuf_xrp: u16,
    #[serde(rename = "tecNO_LINE_INSUF_RESERVE")]
    pub tec_no_line_insuf_reserve: u16,
    #[serde(rename = "tecNO_LINE_REDUNDANT")]
    pub tec_no_line_redundant: u16,
    #[serde(rename = "tecPATH_DRY")]
    pub tec_path_dry: u16,
    #[serde(rename = "tecUNFUNDED")]
    pub tec_unfunded: u16,
    #[serde(rename = "tecNO_ALTERNATIVE_KEY")]
    pub tec_no_alternative_key: u16,
    #[serde(rename = "tecNO_REGULAR_KEY")]
    pub tec_no_regular_key: u16,
    #[serde(rename = "tecOWNERS")]
    pub tec_owners: u16,
    #[serde(rename = "tecNO_ISSUER")]
    pub tec_no_issuer: u16,
    #[serde(rename = "tecNO_AUTH")]
    pub tec_no_auth: u16,
    #[serde(rename = "tecNO_LINE")]
    pub tec_no_line: u16,
    #[serde(rename = "tecINSUFF_FEE")]
    pub tec_insuff_fee: u16,
    #[serde(rename = "tecFROZEN")]
    pub tec_frozen: u16,
    #[serde(rename = "tecNO_TARGET")]
    pub tec_no_target: u16,
    #[serde(rename = "tecNO_PERMISSION")]
    pub tec_no_permission: u16,
    #[serde(rename = "tecNO_ENTRY")]
    pub tec_no_entry: u16,
    #[serde(rename = "tecINSUFFICIENT_RESERVE")]
    pub tec_insufficient_reserve: u16,
    #[serde(rename = "tecNEED_MASTER_KEY")]
    pub tec_need_master_key: u16,
    #[serde(rename = "tecDST_TAG_NEEDED")]
    pub tec_dst_tag_needed: u16,
    #[serde(rename = "tecINTERNAL")]
    pub tec_internal: u16,
    #[serde(rename = "tecOVERSIZE")]
    pub tec_oversize: u16,
    #[serde(rename = "tecCRYPTOCONDITION_ERROR")]
    pub tec_cryptocondition_error: u16,
    #[serde(rename = "tecINVARIANT_FAILED")]
    pub tec_invariant_failed: u16,
    #[serde(rename = "tecEXPIRED")]
    pub tec_expired: u16,
    #[serde(rename = "tecDUPLICATE")]
    pub tec_duplicate: u16,
    #[serde(rename = "tecKILLED")]
    pub tec_killed: u16,
    #[serde(rename = "tecHAS_OBLIGATIONS")]
    pub tec_has_obligations: u16,
    #[serde(rename = "tecTOO_SOON")]
    pub tec_too_soon: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionTypes {
    pub invalid: i16,

    pub payment: u16,
    pub escrow_create: u16,
    pub escrow_finish: u16,
    pub account_set: u16,
    pub escrow_cancel: u16,
    pub set_regular_key: u16,
    pub nick_name_set: u16,
    pub offer_create: u16,
    pub offer_cancel: u16,
    pub contract: u16,
    pub ticket_create: u16,
    pub ticket_cancel: u16,
    pub signer_list_set: u16,
    pub payment_channel_create: u16,
    pub payment_channel_fund: u16,
    pub payment_channel_claim: u16,
    pub check_create: u16,
    pub check_cash: u16,
    pub check_cancel: u16,
    pub deposit_preauth: u16,
    pub trust_set: u16,
    pub account_delete: u16,

    pub enable_amendment: u16,
    pub set_fee: u16,
    #[serde(rename = "UNLModify")]
    pub unl_modify: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Definitions {
    pub types: Types,
    pub ledger_entry_types: LedgerEntryTypes,
    //pub fields: Vec<Vec<FieldInfo>>,
    pub transaction_results: TransactionResults,
    pub transaction_types: TransactionTypes,
}

fn load_definitions() -> Result<Definitions, XRPDefinitionException> {
    Ok(serde_json::from_str(&include_str!("definitions.json"))?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_definitions() {
        let definitions = load_definitions();
        println!("{:?}", definitions);
    }
}
