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
pub struct WholeObject {}

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
    pub const DATA_DRIVEN_TESTS: &str = include_str!("../test_data/data-driven-tests.json");
    pub const CODEC_TEST_FIXTURES: &str = include_str!("../test_data/codec-fixtures.json");
    pub const X_CODEC_TEST_FIXTURES: &str = include_str!("../test_data/x-codec-fixtures.json");

    lazy_static! {
        static ref TEST_CASES: Option<TestDefinitions> =
            Some(serde_json::from_str(DATA_DRIVEN_TESTS).unwrap());
    }

    &TEST_CASES
}

/// Retrieve the field tests.
pub fn load_field_tests() -> &'static Vec<FieldTest> {
    let defintions = _load_tests().as_ref().unwrap();
    &defintions.fields_tests
}

/// Retrieve the field tests.
pub fn load_data_tests(test_type: Option<&str>) -> Vec<ValueTest> {
    let defintions = _load_tests().as_ref().unwrap();

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
