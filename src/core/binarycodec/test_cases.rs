//! Test cases

use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Type {
    pub name: String,
    pub ordinal: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldTest {
    pub type_name: String,
    pub name: String,
    pub nth_of_type: i16,
    pub r#type: i16,
    pub expected_hex: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WholeObject {
    pub tx_json: Value,
    pub fields: Value,
    pub blob_with_no_signing: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueTest {
    pub test_json: Value,
    pub r#type: String,
    pub is_negative: Option<bool>,
    pub is_native: Option<bool>,
    pub type_id: Option<i16>,
    pub expected_hex: Option<String>,
    pub mantissa: Option<String>,
    pub significant_digits: Option<usize>,
    pub exponent: Option<i16>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestDefinitions {
    pub types: Vec<Type>,
    pub fields_tests: Vec<FieldTest>,
    pub whole_objects: Vec<WholeObject>,
    pub values_tests: Vec<ValueTest>,
}

fn _load_tests() -> &'static Option<TestDefinitions> {
    pub const DATA_DRIVEN_TESTS: &str = include_str!("./test_data/data-driven-tests.json");

    lazy_static! {
        static ref TEST_CASES: Option<TestDefinitions> =
            Some(serde_json::from_str(DATA_DRIVEN_TESTS).expect("_load_tests"));
    }

    &TEST_CASES
}

/// Retrieve the field tests.
pub fn load_field_tests() -> &'static Vec<FieldTest> {
    let defintions = _load_tests().as_ref().expect("load_field_tests");
    &defintions.fields_tests
}

/// Retrieve the field tests.
pub fn load_data_tests(test_type: Option<&str>) -> Vec<ValueTest> {
    let defintions = _load_tests().as_ref().expect("load_data_tests");

    if let Some(test) = test_type {
        defintions
            .values_tests
            .clone()
            .into_iter()
            .filter(|vt| vt.r#type == test)
            .collect::<Vec<ValueTest>>()
            .to_vec()
    } else {
        defintions.values_tests.clone()
    }
}

/// A single codec-fixtures entry (used for transactions, accountState, ledgerData).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodecFixtureEntry {
    pub binary: String,
    pub json: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodecFixtures {
    #[serde(rename = "accountState")]
    pub account_state: Vec<CodecFixtureEntry>,
    pub transactions: Vec<CodecFixtureEntry>,
    #[serde(rename = "ledgerData")]
    pub ledger_data: Vec<CodecFixtureEntry>,
}

/// Load codec-fixtures.json.
pub fn load_codec_fixtures() -> &'static CodecFixtures {
    pub const CODEC_FIXTURES: &str = include_str!("./test_data/codec-fixtures.json");

    lazy_static! {
        static ref FIXTURES: CodecFixtures =
            serde_json::from_str(CODEC_FIXTURES).expect("load_codec_fixtures");
    }

    &FIXTURES
}

/// Load whole_objects from data-driven-tests.json for recycle tests.
pub fn load_whole_objects() -> &'static Vec<WholeObject> {
    let definitions = _load_tests().as_ref().expect("load_whole_objects");
    &definitions.whole_objects
}

/// A single x-codec-fixtures entry with rjson (classic) and xjson (X-address) pairs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XCodecFixtureEntry {
    pub rjson: Value,
    pub xjson: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XCodecFixtures {
    pub transactions: Vec<XCodecFixtureEntry>,
}

/// Load x-codec-fixtures.json.
pub fn load_x_codec_fixtures() -> &'static XCodecFixtures {
    pub const X_CODEC_FIXTURES: &str = include_str!("./test_data/x-codec-fixtures.json");

    lazy_static! {
        static ref FIXTURES: XCodecFixtures =
            serde_json::from_str(X_CODEC_FIXTURES).expect("load_x_codec_fixtures");
    }

    &FIXTURES
}
