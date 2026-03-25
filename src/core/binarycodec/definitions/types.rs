//! Maps and helpers providing serialization-related
//! information about fields.

use super::FieldHeader;
use super::FieldInfo;
use super::FieldInstance;
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

/// Dynamic type map: maps type names to type codes.
/// Replaces the old hardcoded `Types` struct so any definitions.json works.
pub type Types = IndexMap<String, i16>;

/// Dynamic ledger entry type map.
pub type LedgerEntryTypes = IndexMap<String, i16>;

/// Dynamic transaction results map.
pub type TransactionResults = IndexMap<String, i16>;

/// Dynamic transaction types map.
pub type TransactionTypes = IndexMap<String, i16>;

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
type DelegatablePermissionsValueMap = IndexMap<String, i32>;
type DelegatablePermissionsNameMap = IndexMap<i32, String>;

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
    delegatable_permissions_value_map: DelegatablePermissionsValueMap,
    delegatable_permissions_name_map: DelegatablePermissionsNameMap,
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
    /// Return an integer representing the given delegatable
    /// permission string.
    fn get_delegatable_permission_code(&self, permission: &str) -> Option<&i32>;
    /// Return string representing the given delegatable
    /// permission code.
    fn get_delegatable_permission_name(&self, code: &i32) -> Option<&String>;
}

/// Build a reverse map (value -> name) from a forward map (name -> value).
fn make_reverse_map(forward: &IndexMap<String, i16>) -> IndexMap<i16, String> {
    let mut reverse = IndexMap::<i16, String>::default();
    for (key, value) in forward {
        reverse.insert(*value, key.to_owned());
    }
    reverse
}

/// Build field info and field header name maps from field definitions.
fn make_field_info_map(
    fields: &[Field],
    types: &TypeValueMap,
) -> (FieldInfoMap, FieldHeaderNameMap) {
    let mut field_info_map = FieldInfoMap::default();
    let mut field_header_name_map = FieldHeaderNameMap::default();
    for field in fields {
        let field_name: &str = &(field.0);
        let field_info: FieldInfo = (field.1).to_owned();
        let field_header = FieldHeader {
            type_code: *types.get(&field_info.r#type).expect("make_field_info_map"),
            field_code: field_info.nth,
        };

        field_info_map.insert(field_name.to_owned(), field_info);
        field_header_name_map.insert(field_header.to_string(), field_name.to_owned());
    }

    (field_info_map, field_header_name_map)
}

impl DefinitionHandler for DefinitionMap {
    fn new(definitions: &Definitions) -> Self {
        let type_value_map = definitions.types.clone();
        let type_name_map = make_reverse_map(&type_value_map);
        let (field_info_map, field_header_name_map) =
            make_field_info_map(&definitions.fields, &type_value_map);
        let transaction_type_value_map = definitions.transaction_types.clone();
        let transaction_type_name_map = make_reverse_map(&transaction_type_value_map);
        let transaction_result_value_map = definitions.transaction_results.clone();
        let transaction_result_name_map = make_reverse_map(&transaction_result_value_map);
        let ledger_entry_type_value_map = definitions.ledger_entry_types.clone();
        let ledger_entry_type_name_map = make_reverse_map(&ledger_entry_type_value_map);

        // Build delegatable permissions map: granular permissions + transaction types (value + 1)
        let mut delegatable_permissions_value_map = DelegatablePermissionsValueMap::default();
        // Granular permissions (hardcoded, matching xrpl.js)
        let granular_permissions: &[(&str, i32)] = &[
            ("TrustlineAuthorize", 65537),
            ("TrustlineFreeze", 65538),
            ("TrustlineUnfreeze", 65539),
            ("AccountDomainSet", 65540),
            ("AccountEmailHashSet", 65541),
            ("AccountMessageKeySet", 65542),
            ("AccountTransferRateSet", 65543),
            ("AccountTickSizeSet", 65544),
            ("PaymentMint", 65545),
            ("PaymentBurn", 65546),
            ("MPTokenIssuanceLock", 65547),
            ("MPTokenIssuanceUnlock", 65548),
        ];
        for (name, code) in granular_permissions {
            delegatable_permissions_value_map.insert(name.to_string(), *code);
        }
        // Transaction types with value + 1
        for (name, code) in &transaction_type_value_map {
            delegatable_permissions_value_map.insert(name.clone(), (*code as i32) + 1);
        }
        let mut delegatable_permissions_name_map = DelegatablePermissionsNameMap::default();
        for (name, code) in &delegatable_permissions_value_map {
            delegatable_permissions_name_map.insert(*code, name.clone());
        }

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
            delegatable_permissions_value_map,
            delegatable_permissions_name_map,
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

    fn get_delegatable_permission_code(&self, permission: &str) -> Option<&i32> {
        self.delegatable_permissions_value_map.get(permission)
    }

    fn get_delegatable_permission_name(&self, code: &i32) -> Option<&String> {
        self.delegatable_permissions_name_map.get(code)
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

/// Return an integer representing the given delegatable
/// permission string.
pub fn get_delegatable_permission_code(permission: &str) -> Option<&i32> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_delegatable_permission_code(permission)
}

/// Return string representing the given delegatable
/// permission code.
pub fn get_delegatable_permission_name(code: &i32) -> Option<&String> {
    let definition_map: &DefinitionMap = load_definition_map();
    definition_map.get_delegatable_permission_name(code)
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
        assert_eq!(get_ledger_entry_type_code("AccountRoot"), Some(&97));
        assert_eq!(get_ledger_entry_type_code("DepositPreauth"), Some(&112));
        assert!(get_ledger_entry_type_code("Nonexistent").is_none());
    }

    #[test]
    fn test_get_ledger_entry_type_name() {
        assert_eq!(
            get_ledger_entry_type_name(&97),
            Some(&"AccountRoot".to_string())
        );
        assert_eq!(
            get_ledger_entry_type_name(&112),
            Some(&"DepositPreauth".to_string())
        );
        assert!(get_ledger_entry_type_name(&9000).is_none());
    }
}
