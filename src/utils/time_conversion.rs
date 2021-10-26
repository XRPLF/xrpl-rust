//! Conversions between the XRP Ledger's 'Ripple Epoch' time and native time
//! data types.

use crate::utils::exceptions::XRPLTimeRangeException;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;

/// The "Ripple Epoch" of 2000-01-01T00:00:00 UTC
pub const RIPPLE_EPOCH: i64 = 946684800;
/// The maximum time that can be expressed on the XRPL
pub const MAX_XRPL_TIME: i64 = i64::pow(2, 32);

/// Ensures time does not exceed max representable on XRPL.
fn _ripple_check_max<T>(time: i64, ok: T) -> Result<T, XRPLTimeRangeException> {
    if !(0..=MAX_XRPL_TIME).contains(&time) {
        Err(XRPLTimeRangeException::UnexpectedTimeOverflow {
            max: MAX_XRPL_TIME,
            found: time,
        })
    } else {
        Ok(ok)
    }
}

/// Convert from XRP Ledger 'Ripple Epoch' time to a UTC datetime.
/// Used internally.
/// See [`chrono::DateTime`]
///
/// [`chrono::DateTime`]: mod@chrono::DateTime
/// ```
pub(crate) fn ripple_time_to_datetime(
    ripple_time: i64,
) -> Result<DateTime<Utc>, XRPLTimeRangeException> {
    _ripple_check_max(ripple_time, Utc.timestamp(ripple_time + RIPPLE_EPOCH, 0))
}

/// Convert from a [`chrono::DateTime`] object to an XRP Ledger
/// 'Ripple Epoch' time.
/// Used internally.
///
/// [`chrono::DateTime`]: mod@chrono::DateTime
/// ```
pub(crate) fn datetime_to_ripple_time(dt: DateTime<Utc>) -> Result<i64, XRPLTimeRangeException> {
    let ripple_time = dt.timestamp() - RIPPLE_EPOCH;
    _ripple_check_max(ripple_time, ripple_time)
}

/// Convert from XRP Ledger 'Ripple Epoch' time to a POSIX-like
/// integer timestamp.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::time_conversion::ripple_time_to_posix;
/// use xrpl::utils::exceptions::XRPLTimeRangeException;
///
/// let posix: Option<i64> = match ripple_time_to_posix(946684801) {
///     Ok(time) => Some(time),
///     Err(e) => match e {
///         XRPLTimeRangeException::InvalidTimeBeforeEpoch { min: _, found: _} => None,
///         XRPLTimeRangeException::UnexpectedTimeOverflow { max: _, found: _ } => None,
///         _ => None,
///     },
/// };
///
/// assert_eq!(Some(1893369601), posix);
/// ```
pub fn ripple_time_to_posix(ripple_time: i64) -> Result<i64, XRPLTimeRangeException> {
    _ripple_check_max(ripple_time, ripple_time + RIPPLE_EPOCH)
}

/// Convert from a POSIX-like timestamp to an XRP Ledger
/// 'Ripple Epoch' time.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use xrpl::utils::time_conversion::posix_to_ripple_time;
/// use xrpl::utils::exceptions::XRPLTimeRangeException;
///
/// let timestamp: Option<i64> = match posix_to_ripple_time(946684801) {
///     Ok(time) => Some(time),
///     Err(e) => match e {
///         XRPLTimeRangeException::InvalidTimeBeforeEpoch { min: _, found: _} => None,
///         XRPLTimeRangeException::UnexpectedTimeOverflow { max: _, found: _ } => None,
///         _ => None,
///     },
/// };
///
/// assert_eq!(Some(1), timestamp);
/// ```
pub fn posix_to_ripple_time(timestamp: i64) -> Result<i64, XRPLTimeRangeException> {
    let ripple_time = timestamp - RIPPLE_EPOCH;
    _ripple_check_max(ripple_time, ripple_time)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ripple_time_to_datetime() {
        let success: DateTime<Utc> = ripple_time_to_datetime(RIPPLE_EPOCH).unwrap();
        assert_eq!(success.timestamp(), RIPPLE_EPOCH + RIPPLE_EPOCH);
    }

    #[test]
    fn test_datetime_to_ripple_time() {
        let success: i64 = datetime_to_ripple_time(Utc.timestamp(RIPPLE_EPOCH, 0)).unwrap();
        assert_eq!(success, 0);
    }

    #[test]
    fn test_ripple_time_to_posix() {
        let success: i64 = ripple_time_to_posix(RIPPLE_EPOCH).unwrap();
        assert_eq!(success, RIPPLE_EPOCH + RIPPLE_EPOCH);
    }

    #[test]
    fn test_posix_to_ripple_time() {
        let success: i64 = posix_to_ripple_time(RIPPLE_EPOCH).unwrap();
        assert_eq!(success, 0);
    }

    #[test]
    fn accept_posix_round_trip() {
        let current_time: i64 = Utc::now().timestamp();
        let ripple_time: i64 = posix_to_ripple_time(current_time).unwrap();
        let round_trip_time: i64 = ripple_time_to_posix(ripple_time).unwrap();

        assert_eq!(current_time, round_trip_time);
    }

    #[test]
    fn accept_datetime_round_trip() {
        let current_time: DateTime<Utc> = Utc.timestamp(Utc::now().timestamp(), 0);
        let ripple_time: i64 = datetime_to_ripple_time(current_time).unwrap();
        let round_trip_time: DateTime<Utc> = ripple_time_to_datetime(ripple_time).unwrap();

        assert_eq!(current_time, round_trip_time);
    }

    #[test]
    fn accept_ripple_epoch() {
        assert_eq!(
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            ripple_time_to_datetime(0).unwrap()
        );
    }

    /// "Ripple Epoch" time starts in the year 2000
    #[test]
    fn accept_datetime_underflow() {
        let datetime: DateTime<Utc> = Utc.ymd(1999, 1, 1).and_hms(0, 0, 0);
        assert!(datetime_to_ripple_time(datetime).is_err())
    }

    /// "Ripple Epoch" time starts in the year 2000
    #[test]
    fn accept_posix_underflow() {
        let datetime: DateTime<Utc> = Utc.ymd(1999, 1, 1).and_hms(0, 0, 0);
        assert!(posix_to_ripple_time(datetime.timestamp()).is_err())
    }

    /// "Ripple Epoch" time's equivalent to the
    /// "Year 2038 problem" is not until 2136
    /// because it uses an *unsigned* 32-bit int
    /// starting 30 years after UNIX time's signed
    /// 32-bit int.
    #[test]
    fn accept_datetime_overflow() {
        let datetime: DateTime<Utc> = Utc.ymd(2137, 1, 1).and_hms(0, 0, 0);
        assert!(datetime_to_ripple_time(datetime).is_err())
    }

    #[test]
    fn accept_posix_overflow() {
        let datetime: DateTime<Utc> = Utc.ymd(2137, 1, 1).and_hms(0, 0, 0);
        assert!(posix_to_ripple_time(datetime.timestamp()).is_err())
    }
}
