//! Base model

use super::XRPLModelResult;

/// A trait that implements basic functions to every model.
pub trait Model {
    /// Collects a models errors and returns the first error that occurs.
    fn get_errors(&self) -> XRPLModelResult<()> {
        Ok(())
    }

    /// Simply forwards the error from `get_errors` if there was one.
    fn validate(&self) -> XRPLModelResult<()> {
        self.get_errors()
    }

    /// Checks if the model is valid.
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}
