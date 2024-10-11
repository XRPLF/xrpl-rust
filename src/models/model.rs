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
        self.get_errors()
    }
}
