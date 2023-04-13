//! Maps and helpers providing serialization-related
//! information about fields.

use crate::core::definitions::FieldHeader;
use crate::core::definitions::FieldInfo;
use crate::core::definitions::FieldInstance;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

type FieldInfoMap = IndexMap<String, FieldInfo>;
type TypeValueMap = IndexMap<String, i16>;
type TypeNameMap = IndexMap<i16, String>;
type FieldHeaderNameMap = IndexMap<String, String>;
type TransactionTypeValueMap = IndexMap<String, i16>;
type TransactionTypeNameMap = IndexMap<i16, String>;
type TransactionResultValueMap = IndexMap<String, i16>;
type TransactionResultNameMap = IndexMap<i16, String>;
type LedgerEntryTypeValueMap = IndexMap<String, i16>;
type LedgerEntryTypeNameMap = IndexMap<i16, String>;

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

/// Loads JSON from the definitions file and converts
/// it to a preferred format. The definitions file contains
/// information required for the XRP Ledger's canonical
/// binary serialization format.
///
/// Serialization:
/// `<https://xrpl.org/serialization.html>`
#[derive(Debug, Clone)]
pub struct DefinitionMap {
    field_info_map: FieldInfoMap,
    type_value_map: TypeValueMap,
    type_name_map: TypeNameMap,
    field_header_name_map: FieldHeaderNameMap,
    transaction_type_value_map: TransactionTypeValueMap,
    transaction_type_name_map: TransactionTypeNameMap,
    transaction_result_value_map: TransactionResultValueMap,
    transaction_result_name_map: TransactionResultNameMap,
    ledger_entry_type_value_map: LedgerEntryTypeValueMap,
    ledger_entry_type_name_map: LedgerEntryTypeNameMap,
}

pub trait DefinitionHandler {
    /// Create a new instance of a definition handler using
    /// a Definitions object.
    fn new(definitions: &Definitions) -> Self;
    /// Get a FieldInfo object from a field name.
    fn get_field_info(&self, key: &str) -> Option<&FieldInfo>;
    /// Returns the serialization data type for the given
    /// field name.
    ///
    /// Serialization Type List:
    /// `<https://xrpl.org/serialization.html#type-list>`
    fn get_field_type_name(&self, field_name: &str) -> Option<&String>;
    /// Returns the type code associated with the given field.
    ///
    /// Serialization Type Codes:
    /// `<https://xrpl.org/serialization.html#type-codes>`
    fn get_field_type_code(&self, field_name: &str) -> Option<&i16>;
    /// Returns the field code associated with the given
    /// field.
    ///
    /// Serialization Field Codes:
    /// `<https://xrpl.org/serialization.html#field-codes>`
    fn get_field_code(&self, field_name: &str) -> Option<i16>;
    /// Returns a FieldHeader object for a field of the given
    /// field name.
    fn get_field_header_from_name(&self, field_name: &str) -> Option<FieldHeader>;
    /// Returns the field name described by the given
    /// FieldHeader object.
    fn get_field_name_from_header(&self, field_header: &FieldHeader) -> Option<&String>;
    /// Return a FieldInstance object for the given field name.
    fn get_field_instance(&self, field_name: &str) -> Option<FieldInstance>;
    /// Return an integer representing the given
    /// transaction type string in an enum.
    fn get_transaction_type_code(&self, transaction_type: &str) -> Option<&i16>;
    /// Return string representing the given transaction
    /// type from the enum.
    fn get_transaction_type_name(&self, transaction_type: &i16) -> Option<&String>;
    /// Return an integer representing the given transaction
    /// result string in an enum.
    fn get_transaction_result_code(&self, transaction_result: &str) -> Option<&i16>;
    /// Return string representing the given transaction result
    /// type from the enum.
    fn get_transaction_result_name(&self, transaction_result: &i16) -> Option<&String>;
    /// Return an integer representing the given ledger entry
    /// type string in an enum.
    fn get_ledger_entry_type_code(&self, ledger_entry_type: &str) -> Option<&i16>;
    /// Return string representing the given ledger entry type
    /// from the enum.
    fn get_ledger_entry_type_name(&self, ledger_entry_type: &i16) -> Option<&String>;
}

trait DefinitionMaker {
    fn _make_type_maps(types: &Types) -> (TypeValueMap, TypeNameMap);
    fn _make_field_info_map(
        fields: &[Field],
        types: &TypeValueMap,
    ) -> (FieldInfoMap, FieldHeaderNameMap);
    fn _make_transaction_type_maps(
        transaction_types: &TransactionTypes,
    ) -> (TransactionTypeValueMap, TransactionTypeNameMap);
    fn _make_transaction_result_maps(
        transaction_types: &TransactionResults,
    ) -> (TransactionResultValueMap, TransactionResultNameMap);
    fn _make_ledger_entry_type_maps(
        types: &LedgerEntryTypes,
    ) -> (LedgerEntryTypeValueMap, LedgerEntryTypeNameMap);
}

impl DefinitionMaker for DefinitionMap {
    fn _make_type_maps(types: &Types) -> (TypeValueMap, TypeNameMap) {
        let json = serde_json::to_string(&types).expect("_make_type_maps_a");
        let v_map: TypeValueMap = serde_json::from_str(&json).expect("_make_type_maps_b");
        let mut n_map: TypeNameMap = TypeNameMap::default();

        for (key, value) in &v_map {
            n_map.insert(*value, key.to_owned());
        }

        (v_map, n_map)
    }

    fn _make_field_info_map(
        fields: &[Field],
        types: &TypeValueMap,
    ) -> (FieldInfoMap, FieldHeaderNameMap) {
        let mut field_info_map = FieldInfoMap::default();
        let mut field_header_name_map = FieldHeaderNameMap::default();

        for field in fields {
            let field_name: &str = &(field.0);
            let field_info: FieldInfo = (field.1).to_owned();
            let field_header = FieldHeader {
                type_code: *types.get(&field_info.r#type).expect("_make_field_info_map"),
                field_code: field_info.nth,
            };

            field_info_map.insert(field_name.to_owned(), field_info);
            field_header_name_map.insert(field_header.to_string(), field_name.to_owned());
        }

        (field_info_map, field_header_name_map)
    }

    fn _make_transaction_type_maps(
        transaction_types: &TransactionTypes,
    ) -> (TransactionTypeValueMap, TransactionTypeNameMap) {
        let json =
            serde_json::to_string(&transaction_types).expect("_make_transaction_type_maps_a");
        let v_map: TransactionTypeValueMap =
            serde_json::from_str(&json).expect("_make_transaction_type_maps_b");
        let mut n_map: TransactionTypeNameMap = TypeNameMap::default();

        for (key, value) in &v_map {
            n_map.insert(*value, key.to_owned());
        }

        (v_map, n_map)
    }

    fn _make_transaction_result_maps(
        transaction_types: &TransactionResults,
    ) -> (TransactionResultValueMap, TransactionResultNameMap) {
        let json =
            serde_json::to_string(&transaction_types).expect("_make_transaction_result_maps_a");
        let v_map: TransactionResultValueMap =
            serde_json::from_str(&json).expect("_make_transaction_result_maps_b");
        let mut n_map: TransactionResultNameMap = TypeNameMap::default();

        for (key, value) in &v_map {
            n_map.insert(*value, key.to_owned());
        }

        (v_map, n_map)
    }

    fn _make_ledger_entry_type_maps(
        types: &LedgerEntryTypes,
    ) -> (LedgerEntryTypeValueMap, LedgerEntryTypeNameMap) {
        let json = serde_json::to_string(&types).expect("_make_ledger_entry_type_maps_a");
        let v_map: TypeValueMap =
            serde_json::from_str(&json).expect("_make_ledger_entry_type_maps_b");
        let mut n_map: TypeNameMap = TypeNameMap::default();

        for (key, value) in &v_map {
            n_map.insert(*value, key.to_owned());
        }

        (v_map, n_map)
    }
}

impl DefinitionHandler for DefinitionMap {
    fn new(definitions: &Definitions) -> Self {
        let (type_value_map, type_name_map) = DefinitionMap::_make_type_maps(&definitions.types);
        let (field_info_map, field_header_name_map) =
            DefinitionMap::_make_field_info_map(&definitions.fields, &type_value_map);
        let (transaction_type_value_map, transaction_type_name_map) =
            DefinitionMap::_make_transaction_type_maps(&definitions.transaction_types);
        let (transaction_result_value_map, transaction_result_name_map) =
            DefinitionMap::_make_transaction_result_maps(&definitions.transaction_results);
        let (ledger_entry_type_value_map, ledger_entry_type_name_map) =
            DefinitionMap::_make_ledger_entry_type_maps(&definitions.ledger_entry_types);

        DefinitionMap {
            field_info_map,
            field_header_name_map,
            type_value_map,
            type_name_map,
            transaction_type_value_map,
            transaction_type_name_map,
            transaction_result_value_map,
            transaction_result_name_map,
            ledger_entry_type_value_map,
            ledger_entry_type_name_map,
        }
    }

    fn get_field_info(&self, key: &str) -> Option<&FieldInfo> {
        self.field_info_map.get(key)
    }

    fn get_field_type_name(&self, field_name: &str) -> Option<&String> {
        let result = self.field_info_map.get(field_name);

        if let Some(value) = result {
            Some(&value.r#type)
        } else {
            None
        }
    }

    fn get_field_type_code(&self, field_name: &str) -> Option<&i16> {
        let result = self.get_field_type_name(field_name);

        if let Some(value) = result {
            self.type_value_map.get(value)
        } else {
            None
        }
    }

    fn get_field_code(&self, field_name: &str) -> Option<i16> {
        let result = self.get_field_info(field_name);
        result.map(|value| value.nth)
    }

    fn get_field_header_from_name(&self, field_name: &str) -> Option<FieldHeader> {
        let type_code_wrap: Option<&i16> = self.get_field_type_code(field_name);
        let field_code_wrap: Option<i16> = self.get_field_code(field_name);

        match (type_code_wrap, field_code_wrap) {
            (Some(type_code), Some(field_code)) => Some(FieldHeader {
                type_code: *type_code,
                field_code,
            }),
            _ => None,
        }
    }

    fn get_field_name_from_header(&self, field_header: &FieldHeader) -> Option<&String> {
        self.field_header_name_map.get(&field_header.to_string())
    }

    fn get_field_instance<'a>(&self, field_name: &str) -> Option<FieldInstance> {
        let field_info_wrap = self.field_info_map.get(field_name);
        let field_header_wrap = self.get_field_header_from_name(field_name);

        match (field_info_wrap, field_header_wrap) {
            (Some(field_info), Some(field_header)) => {
                Some(FieldInstance::new(field_info, field_name, field_header))
            }
            _ => None,
        }
    }

    fn get_transaction_type_code(&self, transaction_type: &str) -> Option<&i16> {
        self.transaction_type_value_map.get(transaction_type)
    }

    fn get_transaction_type_name(&self, transaction_type: &i16) -> Option<&String> {
        self.transaction_type_name_map.get(transaction_type)
    }

    fn get_transaction_result_code(&self, transaction_result: &str) -> Option<&i16> {
        self.transaction_result_value_map.get(transaction_result)
    }

    fn get_transaction_result_name(&self, transaction_result: &i16) -> Option<&String> {
        self.transaction_result_name_map.get(transaction_result)
    }

    fn get_ledger_entry_type_code(&self, ledger_entry_type: &str) -> Option<&i16> {
        self.ledger_entry_type_value_map.get(ledger_entry_type)
    }

    fn get_ledger_entry_type_name(&self, ledger_entry_type: &i16) -> Option<&String> {
        self.ledger_entry_type_name_map.get(ledger_entry_type)
    }
}

fn _load_definitions() -> &'static Option<(Definitions, DefinitionMap)> {
    static JSON: &str = include_str!("definitions.json");

    lazy_static! {
        static ref DEFINITIONS: Option<(Definitions, DefinitionMap)> = {
            let definitions: Definitions = serde_json::from_str(JSON).expect("_load_definitions");
            let definition_map: DefinitionMap = DefinitionMap::new(&definitions);

            Some((definitions, definition_map))
        };
    }

    &DEFINITIONS
}

/// Retrieve the definition map.
pub fn load_definition_map() -> &'static DefinitionMap {
    let (_, map) = _load_definitions().as_ref().expect("load_definition_map");
    map
}

/// Returns the serialization data type for the
/// given field name.
///
/// Serialization Type List:
/// `<https://xrpl.org/serialization.html#type-list>`
pub fn get_field_type_name(field_name: &str) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_field_type_name(field_name)
}

/// Returns the type code associated with the
/// given field.
///
/// Serialization Type Codes:
/// `<https://xrpl.org/serialization.html#type-codes>`
pub fn get_field_type_code(field_name: &str) -> Option<&i16> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_field_type_code(field_name)
}

/// Returns the field code associated with the
/// given field.
///
/// Serialization Field Codes:
/// `<https://xrpl.org/serialization.html#field-codes>`
pub fn get_field_code(field_name: &str) -> Option<i16> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_field_code(field_name)
}

/// Returns a FieldHeader object for a field of
/// the given field name.
pub fn get_field_header_from_name(field_name: &str) -> Option<FieldHeader> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_field_header_from_name(field_name)
}

/// Returns the field name described by the
/// given FieldHeader object.
pub fn get_field_name_from_header(field_header: &FieldHeader) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_field_name_from_header(field_header)
}

/// Return a FieldInstance object for the given
/// field name.
pub fn get_field_instance(field_name: &str) -> Option<FieldInstance> {
    let definition_map: &DefinitionMap = load_definition_map();

    definition_map.get_field_instance(field_name)
}

/// Return an integer representing the given
/// transaction type string in an enum.
pub fn get_transaction_type_code(transaction_type: &str) -> Option<&i16> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_transaction_type_code(transaction_type)
}

/// Return an integer representing the given
/// transaction type string in an enum.
pub fn get_transaction_type_name(transaction_type: &i16) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_transaction_type_name(transaction_type)
}

/// Return an integer representing the given
/// transaction result string in an enum.
pub fn get_transaction_result_code(transaction_result_type: &str) -> Option<&i16> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_transaction_result_code(transaction_result_type)
}

/// Return string representing the given transaction
/// result type from the enum.
pub fn get_transaction_result_name(transaction_result_type: &i16) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_transaction_result_name(transaction_result_type)
}

/// Return an integer representing the given ledger
/// entry type string in an enum.
pub fn get_ledger_entry_type_code(ledger_entry_type: &str) -> Option<&i16> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_ledger_entry_type_code(ledger_entry_type)
}

/// Return an integer representing the given ledger
/// entry type string in an enum.
pub fn get_ledger_entry_type_name(ledger_entry_type: &i16) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_ledger_entry_type_name(ledger_entry_type)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_definitions() {
        assert!(!_load_definitions().is_none());
    }

    #[test]
    fn test_get_field_type_name() {
        assert_eq!(
            get_field_type_name("HighLimit"),
            Some(&"Amount".to_string())
        );
    }

    #[test]
    fn test_get_field_type_code() {
        assert_eq!(get_field_type_code("HighLimit"), Some(&6));
        assert_eq!(get_field_type_code("Generic"), Some(&-2));
    }

    #[test]
    fn test_get_field_code() {
        assert_eq!(get_field_code("HighLimit"), Some(7));
        assert_eq!(get_field_code("Generic"), Some(0));
        assert_eq!(get_field_code("Invalid"), Some(-1));
        assert!(get_field_code("Nonexistent").is_none());
    }

    #[test]
    fn test_get_field_header_from_name() {
        let field_header = get_field_header_from_name("Generic").unwrap();

        assert_eq!(-2, field_header.type_code);
        assert_eq!(0, field_header.field_code);
    }

    #[test]
    fn test_get_field_name_from_header() {
        let field_header = FieldHeader {
            type_code: -2,
            field_code: 0,
        };

        assert_eq!(
            get_field_name_from_header(&field_header),
            Some(&"Generic".to_string())
        );
    }

    #[test]
    fn test_get_field_instance() {
        let field_header = FieldHeader {
            type_code: -2,
            field_code: 0,
        };

        let field_info = FieldInfo {
            nth: 0,
            is_vl_encoded: false,
            is_serialized: false,
            is_signing_field: false,
            r#type: "Unknown".to_string(),
        };

        let field_instance = FieldInstance::new(&field_info, "Generic", field_header);
        let test_field_instance = get_field_instance("Generic");

        assert!(test_field_instance.is_some());

        let test_field_instance = test_field_instance.unwrap();

        assert_eq!(
            field_instance.header.type_code,
            test_field_instance.header.type_code
        );
    }

    #[test]
    fn test_get_transaction_type_code() {
        assert_eq!(get_transaction_type_code("Invalid"), Some(&-1));
        assert_eq!(get_transaction_type_code("OfferCancel"), Some(&8));
        assert!(get_transaction_type_code("Nonexistent").is_none());
    }

    #[test]
    fn test_get_transaction_type_name() {
        assert_eq!(get_transaction_type_name(&-1), Some(&"Invalid".to_string()));
        assert_eq!(get_transaction_type_name(&0), Some(&"Payment".to_string()));
        assert!(get_transaction_type_name(&9000).is_none());
    }

    #[test]
    fn test_get_transaction_result_code() {
        assert_eq!(get_transaction_result_code("telLOCAL_ERROR"), Some(&-399));
        assert_eq!(
            get_transaction_result_code("temCANNOT_PREAUTH_SELF"),
            Some(&-267)
        );
        assert!(get_transaction_result_code("Nonexistent").is_none());
    }

    #[test]
    fn test_get_transaction_result_name() {
        assert_eq!(
            get_transaction_result_name(&-399),
            Some(&"telLOCAL_ERROR".to_string())
        );
        assert_eq!(
            get_transaction_result_name(&-267),
            Some(&"temCANNOT_PREAUTH_SELF".to_string()),
        );
        assert!(get_transaction_result_name(&9000).is_none());
    }

    #[test]
    fn test_get_ledger_entry_type_code() {
        assert_eq!(get_ledger_entry_type_code("Any"), Some(&-3));
        assert_eq!(get_ledger_entry_type_code("DepositPreauth"), Some(&112));
        assert!(get_ledger_entry_type_code("Nonexistent").is_none());
    }

    #[test]
    fn test_get_ledger_entry_type_name() {
        assert_eq!(get_ledger_entry_type_name(&-3), Some(&"Any".to_string()));
        assert_eq!(
            get_ledger_entry_type_name(&112),
            Some(&"DepositPreauth".to_string())
        );
        assert!(get_ledger_entry_type_name(&9000).is_none());
    }
}
