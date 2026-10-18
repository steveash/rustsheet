//! The clock a workbook reads the current time from (SPEC-0008).
//!
//! NOW, TODAY, year-less date text (`"7/1"`), the creation timestamp of a new workbook and the
//! exporter's `dcterms:modified` all read [`Model::now_unix_millis`](crate::Model), which asks
//! the model's [`Clock`]. The clock is per model, so two workbooks on one thread can have
//! different clocks and a workbook keeps its clock when it moves to another thread.

/// 1900-01-01T00:00:00Z, the first instant of Excel's calendar, in ms since the Unix epoch.
const MIN_UNIX_MILLIS: i64 = -2_208_988_800_000;
/// 9999-12-31T23:59:59.999Z, the last instant of Excel's calendar, in ms since the Unix epoch.
const MAX_UNIX_MILLIS: i64 = 253_402_300_799_999;

/// Where a workbook reads the current time from (NOW, TODAY, year-less date text, timestamps).
///
/// Either the system time (the default) or a fixed instant in Excel's calendar. The field is
/// private, so a `Clock` always holds an instant every reader can convert without an error
/// path.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Clock(Kind);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Kind {
    #[default]
    System,
    Fixed(i64),
}

/// A fixed clock instant outside Excel's calendar.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("clock instant {0} ms is outside 1900-01-01..=9999-12-31 UTC")]
pub struct ClockError(pub i64);

impl Clock {
    /// The system clock: each read takes the host's current time.
    #[must_use]
    pub const fn system() -> Self {
        Self(Kind::System)
    }

    /// A fixed instant, in milliseconds since 1970-01-01T00:00:00Z.
    ///
    /// # Errors
    /// [`ClockError`] when the instant's UTC date is outside 1900-01-01 through 9999-12-31.
    pub fn fixed_unix_millis(ms: i64) -> Result<Self, ClockError> {
        if (MIN_UNIX_MILLIS..=MAX_UNIX_MILLIS).contains(&ms) {
            Ok(Self(Kind::Fixed(ms)))
        } else {
            Err(ClockError(ms))
        }
    }

    /// The fixed instant, or `None` for the system clock.
    #[must_use]
    pub fn fixed(self) -> Option<i64> {
        match self.0 {
            Kind::System => None,
            Kind::Fixed(ms) => Some(ms),
        }
    }

    /// The current instant in ms since the Unix epoch, always within Excel's calendar.
    #[must_use]
    pub fn now_unix_millis(self) -> i64 {
        match self.0 {
            Kind::System => system_now_unix_millis(),
            Kind::Fixed(ms) => ms,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn system_now_unix_millis() -> i64 {
    system_unix_millis(std::time::SystemTime::now())
}

/// The browser's `Date.now()`, clamped like [`system_unix_millis`]; a non-finite value gives
/// the lower bound.
#[cfg(target_arch = "wasm32")]
fn system_now_unix_millis() -> i64 {
    let ms = js_sys::Date::now();
    if ms.is_finite() {
        // `as` saturates a finite float to the `i64` range; the clamp does the rest.
        (ms.floor() as i64).clamp(MIN_UNIX_MILLIS, MAX_UNIX_MILLIS)
    } else {
        MIN_UNIX_MILLIS
    }
}

/// `t` in ms since the Unix epoch (floored), clamped to Excel's calendar. A host clock outside
/// 1900..=9999 is a misconfiguration Excel cannot run with; clamping it instead of failing is a
/// directive-5 choice (no panic on a bad system clock, SPEC-0008).
#[cfg(not(target_arch = "wasm32"))]
fn system_unix_millis(t: std::time::SystemTime) -> i64 {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis())
            .unwrap_or(MAX_UNIX_MILLIS)
            .min(MAX_UNIX_MILLIS),
        Err(before) => {
            let before = before.duration();
            // Floor, so a time a fraction of a millisecond before an instant reads as the
            // millisecond before it, as it does after the epoch.
            let partial = u128::from(before.subsec_nanos() % 1_000_000 != 0);
            i64::try_from(before.as_millis().saturating_add(partial))
                .ok()
                .and_then(i64::checked_neg)
                .unwrap_or(MIN_UNIX_MILLIS)
                .max(MIN_UNIX_MILLIS)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn default_is_the_system_clock() {
        assert_eq!(Clock::default(), Clock::system());
        assert_eq!(Clock::system().fixed(), None);
        let host = system_unix_millis(SystemTime::now());
        let read = Clock::system().now_unix_millis();
        assert!((read - host).abs() < 5_000, "{read} vs {host}");
    }

    #[test]
    fn fixed_accepts_both_bounds_and_rejects_beyond() {
        for ms in [MIN_UNIX_MILLIS, 0, MAX_UNIX_MILLIS] {
            let clock = Clock::fixed_unix_millis(ms).unwrap();
            assert_eq!(clock.fixed(), Some(ms));
            assert_eq!(clock.now_unix_millis(), ms);
        }
        for ms in [MIN_UNIX_MILLIS - 1, MAX_UNIX_MILLIS + 1, i64::MIN, i64::MAX] {
            assert_eq!(Clock::fixed_unix_millis(ms), Err(ClockError(ms)));
        }
    }

    #[test]
    fn system_time_converts_and_clamps() {
        assert_eq!(system_unix_millis(UNIX_EPOCH), 0);
        assert_eq!(
            system_unix_millis(UNIX_EPOCH - Duration::from_millis(1_500)),
            -1_500
        );
        assert_eq!(
            system_unix_millis(UNIX_EPOCH - Duration::from_micros(1_500)),
            -2
        );
        assert_eq!(
            system_unix_millis(UNIX_EPOCH + Duration::from_micros(1_500)),
            1
        );
        let four_hundred_years = Duration::from_secs(400 * 366 * 86_400);
        if let Some(t) = UNIX_EPOCH.checked_sub(four_hundred_years) {
            assert_eq!(system_unix_millis(t), MIN_UNIX_MILLIS);
        }
        if let Some(t) = UNIX_EPOCH.checked_add(four_hundred_years * 25) {
            assert_eq!(system_unix_millis(t), MAX_UNIX_MILLIS);
        }
        if let Some(t) = UNIX_EPOCH.checked_add(Duration::MAX) {
            assert_eq!(system_unix_millis(t), MAX_UNIX_MILLIS);
        }
        if let Some(t) = UNIX_EPOCH.checked_sub(Duration::MAX) {
            assert_eq!(system_unix_millis(t), MIN_UNIX_MILLIS);
        }
    }
}
