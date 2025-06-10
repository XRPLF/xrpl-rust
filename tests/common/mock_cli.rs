#![cfg(all(not(feature = "std"), feature = "cli", test))]

use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

// Mock output capture for no_std testing
thread_local! {
    static CAPTURED_OUTPUT: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

// Mock println! for testing
#[macro_export]
macro_rules! mock_println {
    ($($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        let bytes = s.as_bytes();
        $crate::common::mock_cli::CAPTURED_OUTPUT.with(|output| {
            output.borrow_mut().extend_from_slice(bytes);
            output.borrow_mut().push(b'\n');
        });
    }};
}

// Function to get captured output
pub fn get_captured_output() -> String {
    CAPTURED_OUTPUT.with(|output| {
        let bytes = output.borrow().clone();
        String::from_utf8(bytes).unwrap_or_default()
    })
}

// Function to clear captured output
pub fn clear_captured_output() {
    CAPTURED_OUTPUT.with(|output| {
        output.borrow_mut().clear();
    });
}

// Mock client for testing
pub struct MockJsonRpcClient;

impl MockJsonRpcClient {
    pub fn connect(_url: url::Url) -> Self {
        MockJsonRpcClient
    }
    
    pub fn request(&self, _request: xrpl::models::Request) -> Result<xrpl::models::Response, xrpl::clients::exceptions::XRPLClientException> {
        // Return a mock response
        Ok(xrpl::models::Response::default())
    }
}