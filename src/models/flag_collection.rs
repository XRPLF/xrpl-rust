use core::convert::TryFrom;

use alloc::vec::Vec;
use anyhow::Result;
use derive_new::new;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};

use crate::{models::XRPLFlagsException, Err};

/// Represents the type of flags when the XRPL model has no flags.
#[derive(
    Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Display, AsRefStr, EnumIter, Copy,
)]
pub enum NoFlags {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, new)]
pub struct FlagCollection<T>(pub(crate) Vec<T>)
where
    T: IntoEnumIterator;

impl<T> Iterator for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T> Default for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    fn default() -> Self {
        FlagCollection(Vec::new())
    }
}

impl<T> From<Vec<T>> for FlagCollection<T>
where
    T: IntoEnumIterator,
{
    fn from(flags: Vec<T>) -> Self {
        FlagCollection(flags)
    }
}

impl<T> TryFrom<u32> for FlagCollection<T>
where
    T: IntoEnumIterator + Serialize,
{
    type Error = anyhow::Error;

    fn try_from(flags: u32) -> Result<Self> {
        let mut flag_collection = Vec::new();
        for flag in T::iter() {
            let flag_as_u32 = flag_to_u32(&flag)?;
            if flags & flag_as_u32 == flag_as_u32 {
                flag_collection.push(flag);
            }
        }
        Ok(FlagCollection::new(flag_collection))
    }
}

impl<T> TryFrom<FlagCollection<T>> for u32
where
    T: IntoEnumIterator + Serialize,
{
    type Error = anyhow::Error;

    fn try_from(flag_collection: FlagCollection<T>) -> Result<Self> {
        let mut flags = 0;
        for flag in flag_collection {
            let flag_as_u32 = flag_to_u32(&flag)?;
            flags |= flag_as_u32;
        }
        Ok(flags)
    }
}

fn flag_to_u32<T>(flag: &T) -> Result<u32>
where
    T: Serialize,
{
    match serde_json::to_string(flag) {
        Ok(flag_as_string) => match flag_as_string.parse::<u32>() {
            Ok(flag_as_u32) => Ok(flag_as_u32),
            Err(_error) => Err!(XRPLFlagsException::CannotConvertFlagToU32),
        },
        Err(_error) => Err!(XRPLFlagsException::CannotConvertFlagToU32),
    }
}
