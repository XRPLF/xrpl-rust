//! Base model

use anyhow::Result;

/// A trait that implements basic functions to every model.
pub trait Model {
    /// Collects a models errors and returns the first error that occurs.
    fn get_errors(&self) -> Result<()> {
        Ok(())
    }

    /// Simply forwards the error from `get_errors` if there was one.
    fn validate(&self) -> Result<()> {
        match self.get_errors() {
            Ok(_no_error) => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Returns whether the structure is valid.
    fn is_valid(&self) -> bool {
        match self.get_errors() {
            Ok(_no_error) => true,
            Err(_error) => false,
        }
    }
}
