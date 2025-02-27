use core::convert::TryFrom;

use alloc::vec::Vec;
use derive_new::new;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter};

use super::{XRPLModelException, XRPLModelResult};

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
    type Error = XRPLModelException;

    fn try_from(flags: u32) -> XRPLModelResult<Self> {
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

impl<T> TryFrom<Option<u32>> for FlagCollection<T>
where
    T: IntoEnumIterator + Serialize,
{
    type Error = XRPLModelException;

    fn try_from(flags: Option<u32>) -> XRPLModelResult<Self> {
        match flags {
            Some(flags) => FlagCollection::try_from(flags),
            None => Ok(FlagCollection::default()),
        }
    }
}

impl<T> TryFrom<FlagCollection<T>> for u32
where
    T: IntoEnumIterator + Serialize,
{
    type Error = XRPLModelException;

    fn try_from(flag_collection: FlagCollection<T>) -> XRPLModelResult<Self> {
        let mut flags = 0;
        for flag in flag_collection {
            let flag_as_u32 = flag_to_u32(&flag)?;
            flags |= flag_as_u32;
        }
        Ok(flags)
    }
}

impl<T> core::fmt::Display for FlagCollection<T>
where
    T: IntoEnumIterator + Serialize,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut flags = 0;
        for flag in &self.0 {
            let flag_as_u32 = flag_to_u32(flag).unwrap();
            flags |= flag_as_u32;
        }
        write!(f, "{}", flags)
    }
}

impl<T> FlagCollection<T>
where
    T: IntoEnumIterator + Serialize,
{
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn flag_to_u32<T>(flag: &T) -> XRPLModelResult<u32>
where
    T: Serialize,
{
    Ok(serde_json::to_string(flag)?.parse::<u32>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ledger::objects::account_root::AccountRootFlag;

    #[test]
    fn test_flag_collection_conversion() {
        // Test u32 to FlagCollection
        let flags_u32: u32 = 0x00800000; // LsfDefaultRipple
        let flag_collection = FlagCollection::<AccountRootFlag>::try_from(flags_u32).unwrap();
        assert_eq!(
            flag_collection.0,
            alloc::vec![AccountRootFlag::LsfDefaultRipple]
        );

        // Test FlagCollection to u32
        let flags_back: u32 = u32::try_from(flag_collection).unwrap();
        assert_eq!(flags_back, flags_u32);

        // Test multiple flags
        let flags_u32: u32 = 0x00800000 | 0x01000000; // LsfDefaultRipple | LsfDepositAuth
        let flag_collection = FlagCollection::<AccountRootFlag>::try_from(flags_u32).unwrap();
        assert!(flag_collection
            .0
            .contains(&AccountRootFlag::LsfDefaultRipple));
        assert!(flag_collection.0.contains(&AccountRootFlag::LsfDepositAuth));

        let flags_back: u32 = u32::try_from(flag_collection).unwrap();
        assert_eq!(flags_back, flags_u32);
    }
}
