//! Maps and helpers providing serialization-related
//! information about fields.

use crate::core::binarycodec::definitions::field_info::FieldInfo;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

type FieldInfoMap = IndexMap<String, FieldInfo>;
type FieldHeaderMap = IndexMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Types {
    pub validation: i16,
    pub done: i16,
    pub hash_128: i16,
    pub blob: i16,
    #[serde(rename = "AccountID")]
    pub account_id: i16,
    pub amount: i16,
    pub hash_256: i16,
    pub u_int_8: i16,
    pub vector_256: i16,
    pub serialized_dict: i16,
    pub unknown: i16,
    pub transaction: i16,
    pub hash_160: i16,
    pub path_set: i16,
    pub ledger_entry: i16,
    pub u_int_16: i16,
    pub not_present: i16,
    pub u_int_64: i16,
    pub u_int_32: i16,
    pub serialized_list: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LedgerEntryTypes {
    pub any: i16,
    pub child: i16,
    pub invalid: i16,
    pub account_root: i16,
    pub directory_node: i16,
    pub ripple_state: i16,
    pub ticket: i16,
    pub signer_list: i16,
    pub offer: i16,
    pub ledger_hashes: i16,
    pub amendments: i16,
    pub fee_settings: i16,
    pub escrow: i16,
    pub pay_channel: i16,
    pub deposit_preauth: i16,
    pub check: i16,
    pub nickname: i16,
    pub contract: i16,
    pub generator_map: i16,
    #[serde(rename = "NegativeUNL")]
    pub negative_unl: i16,
}

/// =(
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub tes_success: i16,

    #[serde(rename = "tecCLAIM")]
    pub tec_claim: i16,
    #[serde(rename = "tecPATH_PARTIAL")]
    pub tec_path_partial: i16,
    #[serde(rename = "tecUNFUNDED_ADD")]
    pub tec_unfunded_add: i16,
    #[serde(rename = "tecUNFUNDED_OFFER")]
    pub tec_unfunded_offer: i16,
    #[serde(rename = "tecUNFUNDED_PAYMENT")]
    pub tec_unfunded_payment: i16,
    #[serde(rename = "tecFAILED_PROCESSING")]
    pub tec_failed_processing: i16,
    #[serde(rename = "tecDIR_FULL")]
    pub tec_dir_full: i16,
    #[serde(rename = "tecINSUF_RESERVE_LINE")]
    pub tec_insuf_reserve_line: i16,
    #[serde(rename = "tecINSUF_RESERVE_OFFER")]
    pub tec_insuf_reserve_offer: i16,
    #[serde(rename = "tecNO_DST")]
    pub tec_no_dst: i16,
    #[serde(rename = "tecNO_DST_INSUF_XRP")]
    pub tec_no_dst_insuf_xrp: i16,
    #[serde(rename = "tecNO_LINE_INSUF_RESERVE")]
    pub tec_no_line_insuf_reserve: i16,
    #[serde(rename = "tecNO_LINE_REDUNDANT")]
    pub tec_no_line_redundant: i16,
    #[serde(rename = "tecPATH_DRY")]
    pub tec_path_dry: i16,
    #[serde(rename = "tecUNFUNDED")]
    pub tec_unfunded: i16,
    #[serde(rename = "tecNO_ALTERNATIVE_KEY")]
    pub tec_no_alternative_key: i16,
    #[serde(rename = "tecNO_REGULAR_KEY")]
    pub tec_no_regular_key: i16,
    #[serde(rename = "tecOWNERS")]
    pub tec_owners: i16,
    #[serde(rename = "tecNO_ISSUER")]
    pub tec_no_issuer: i16,
    #[serde(rename = "tecNO_AUTH")]
    pub tec_no_auth: i16,
    #[serde(rename = "tecNO_LINE")]
    pub tec_no_line: i16,
    #[serde(rename = "tecINSUFF_FEE")]
    pub tec_insuff_fee: i16,
    #[serde(rename = "tecFROZEN")]
    pub tec_frozen: i16,
    #[serde(rename = "tecNO_TARGET")]
    pub tec_no_target: i16,
    #[serde(rename = "tecNO_PERMISSION")]
    pub tec_no_permission: i16,
    #[serde(rename = "tecNO_ENTRY")]
    pub tec_no_entry: i16,
    #[serde(rename = "tecINSUFFICIENT_RESERVE")]
    pub tec_insufficient_reserve: i16,
    #[serde(rename = "tecNEED_MASTER_KEY")]
    pub tec_need_master_key: i16,
    #[serde(rename = "tecDST_TAG_NEEDED")]
    pub tec_dst_tag_needed: i16,
    #[serde(rename = "tecINTERNAL")]
    pub tec_internal: i16,
    #[serde(rename = "tecOVERSIZE")]
    pub tec_oversize: i16,
    #[serde(rename = "tecCRYPTOCONDITION_ERROR")]
    pub tec_cryptocondition_error: i16,
    #[serde(rename = "tecINVARIANT_FAILED")]
    pub tec_invariant_failed: i16,
    #[serde(rename = "tecEXPIRED")]
    pub tec_expired: i16,
    #[serde(rename = "tecDUPLICATE")]
    pub tec_duplicate: i16,
    #[serde(rename = "tecKILLED")]
    pub tec_killed: i16,
    #[serde(rename = "tecHAS_OBLIGATIONS")]
    pub tec_has_obligations: i16,
    #[serde(rename = "tecTOO_SOON")]
    pub tec_too_soon: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TransactionTypes {
    pub invalid: i16,

    pub payment: i16,
    pub escrow_create: i16,
    pub escrow_finish: i16,
    pub account_set: i16,
    pub escrow_cancel: i16,
    pub set_regular_key: i16,
    pub nick_name_set: i16,
    pub offer_create: i16,
    pub offer_cancel: i16,
    pub contract: i16,
    pub ticket_create: i16,
    pub ticket_cancel: i16,
    pub signer_list_set: i16,
    pub payment_channel_create: i16,
    pub payment_channel_fund: i16,
    pub payment_channel_claim: i16,
    pub check_create: i16,
    pub check_cash: i16,
    pub check_cancel: i16,
    pub deposit_preauth: i16,
    pub trust_set: i16,
    pub account_delete: i16,

    pub enable_amendment: i16,
    pub set_fee: i16,
    #[serde(rename = "UNLModify")]
    pub unl_modify: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field(pub String, pub FieldInfo);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct Definitions {
    pub types: Types,
    pub ledger_entry_types: LedgerEntryTypes,
    pub fields: Vec<Field>,
    pub transaction_results: TransactionResults,
    pub transaction_types: TransactionTypes,
}

unsafe fn _load_definitions<'a>() -> &'a Option<Definitions> {
    static JSON: &str = include_str!("definitions.json");
    static mut DEFINITIONS: Option<Definitions> = None;

    if DEFINITIONS.is_none() {
        DEFINITIONS = Some(serde_json::from_str(JSON).unwrap());
    };

    &DEFINITIONS
}

unsafe fn _field_info_map<'a>() -> &'a Option<FieldInfoMap> {
    static mut FIELD_INFO_MAP: Option<FieldInfoMap> = None;

    if FIELD_INFO_MAP.is_none() {
        let definitions: &Definitions = _load_definitions().as_ref().unwrap();
        let mut map = FieldInfoMap::default();

        for field in &definitions.fields {
            map.insert((field.0).to_owned(), (field.1).to_owned());
        }

        FIELD_INFO_MAP = Some(map);
    }

    &FIELD_INFO_MAP
}

fn _field_entry_val(param: &str) -> Option<i16> {
    let definitions: &Definitions = unsafe { _load_definitions().as_ref().unwrap() };

    match param {
        "Validation" => Some(definitions.types.validation),
        "Hash128" => Some(definitions.types.hash_128),
        "Hash256" => Some(definitions.types.hash_256),
        "Blob" => Some(definitions.types.blob),
        "AccountID" => Some(definitions.types.account_id),
        "Amount" => Some(definitions.types.amount),
        "UInt8" => Some(definitions.types.u_int_8),
        "Vector256" => Some(definitions.types.vector_256),
        "SerializedDict" => Some(definitions.types.serialized_dict),
        "Unknown" => Some(definitions.types.unknown),
        "Transaction" => Some(definitions.types.transaction),
        "Hash160" => Some(definitions.types.hash_160),
        "PathSet" => Some(definitions.types.path_set),
        "LedgerEntry" => Some(definitions.types.ledger_entry),
        "UInt16" => Some(definitions.types.u_int_16),
        "UInt64" => Some(definitions.types.u_int_64),
        "UInt32" => Some(definitions.types.u_int_32),
        "SerializedList" => Some(definitions.types.serialized_list),
        _ => None,
    }
}

/// Returns the serialization data type for the
/// given field name.
///
/// Serialization Type List:
/// https://xrpl.org/serialization.html#type-list
pub fn get_field_type_name(field_name: &str) -> Option<String> {
    let field_info_map: &FieldInfoMap = unsafe { _field_info_map().as_ref().unwrap() };
    let result = field_info_map.get(field_name);

    if result.is_none() {
        None
    } else {
        Some(result.unwrap().r#type.to_owned())
    }
}

/// Returns the type code associated with the
/// given field.
///
/// Serialization Type Codes:
/// https://xrpl.org/serialization.html#type-codes
pub fn get_field_type_code(field_name: &str) -> Option<i16> {
    let field_type_name = get_field_type_name(field_name);

    if field_type_name.is_none() {
        None
    } else {
        let field_type_code = _field_entry_val(&field_type_name.unwrap());
        field_type_code
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_definitions() {
        unsafe {
            assert!(!_load_definitions().is_none());
        }
    }

    #[test]
    fn test_field_entry_val() {
        assert_eq!(10003, _field_entry_val("Validation").unwrap());
        assert_eq!(-2, _field_entry_val("Unknown").unwrap());
        assert!(_field_entry_val("Nonexistent").is_none());
    }

    #[test]
    fn test_get_field_type_name() {
        let field_type_name: Option<String> = get_field_type_name("HighLimit");

        assert!(!field_type_name.is_none());
        assert_eq!("Amount", field_type_name.unwrap());
    }

    #[test]
    fn test_get_field_type_code() {
        assert_eq!(6, get_field_type_code("HighLimit").unwrap());
        assert_eq!(-2, get_field_type_code("Generic").unwrap());
    }
}
