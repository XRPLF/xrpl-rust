use alloc::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Response format for the random command, which provides a random number to
/// be used as a source of entropy for random number generation by clients.
///
/// A successful result contains a 256-bit hex value that can be used for
/// random number generation purposes.
///
/// See Random:
/// `<https://xrpl.org/random.html#random>`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Random<'a> {
    /// A random 256-bit value represented as a hexadecimal string.
    pub random: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_random_deserialization() {
        let json_data = json!({
            "random": "8ED765AEBBD6767603C2C9375B2679AEC76E6A8133EF59F04F9FC1AAA70E41AF"
        });

        let random: Random = serde_json::from_value(json_data).unwrap();
        assert_eq!(
            random.random,
            "8ED765AEBBD6767603C2C9375B2679AEC76E6A8133EF59F04F9FC1AAA70E41AF"
        );
    }

    #[test]
    fn test_random_serialization() {
        let random = Random {
            random: Cow::Borrowed(
                "8ED765AEBBD6767603C2C9375B2679AEC76E6A8133EF59F04F9FC1AAA70E41AF",
            ),
        };

        let json = serde_json::to_value(&random).unwrap();
        assert_eq!(
            json,
            json!({
                "random": "8ED765AEBBD6767603C2C9375B2679AEC76E6A8133EF59F04F9FC1AAA70E41AF"
            })
        );
    }
}
