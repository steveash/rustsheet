//! Elapsed `[h]`/`[m]`/`[s]` token text and fractional seconds (`s.000`, `[s].000`).

use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;

/// The unit of an elapsed `[h]`/`[m]`/`[s]` token.
#[derive(Clone, Copy)]
pub(super) enum ElapsedUnit {
    Hour,
    Minute,
    Second,
}

/// A non-negative serial counted in the shown fraction of a second (`.0`, `.00`, `.000`). Excel
/// rounds the whole value to that precision and splits it from there, so the clock, elapsed and
/// fraction tokens agree: `[h]:m:s.000` on 3.14159 is "75:23:53.376", `[hh]:mm:ss.0` "75:23:53.4",
/// `[s].000` "271433.376" (an elapsed count before a fraction truncates instead of rounding);
/// poi/ElapsedFormatTests A2-A7, number-formats.md "Elapsed-time tokens".
pub(super) struct FractionalSeconds {
    /// The value in units of `1/scale` second, rounded.
    units: f64,
    scale: f64,
    digits: usize,
}

impl FractionalSeconds {
    /// `None` for more digits than a format may show. The caller has rejected serials past
    /// 9999-12-31, so the units stay far below 2^53 and count exactly.
    pub(super) fn new(value: f64, digits: usize) -> Option<Self> {
        let scale = [1.0, 10.0, 100.0, 1000.0].get(digits).copied()?;
        Some(Self {
            units: (value * 86400.0 * scale).round(),
            scale,
            digits,
        })
    }

    /// Hours, minutes and seconds of the time of day after rounding.
    pub(super) fn clock(&self) -> (f64, f64, f64) {
        let seconds = (self.units % (86400.0 * self.scale) / self.scale).floor();
        (
            (seconds / 3600.0).floor(),
            (seconds % 3600.0 / 60.0).floor(),
            seconds % 60.0,
        )
    }

    /// The whole days after rounding: a time just short of midnight rolls into the next date.
    pub(super) fn days(&self) -> i64 {
        (self.units / (86400.0 * self.scale)).floor() as i64
    }

    /// The whole elapsed count of `unit` after rounding.
    fn elapsed(&self, unit: ElapsedUnit) -> f64 {
        let seconds = (self.units / self.scale).floor();
        match unit {
            ElapsedUnit::Hour => (seconds / 3600.0).floor(),
            ElapsedUnit::Minute => (seconds / 60.0).floor(),
            ElapsedUnit::Second => seconds,
        }
    }

    /// The fraction digits, zero-padded (`376`, `4`, `00`).
    pub(super) fn digits_text(&self) -> String {
        format!(
            "{:0>width$.0}",
            self.units % self.scale,
            width = self.digits
        )
    }
}

/// The clock, day and elapsed-count inputs of one date format, all from the serial rounded to the
/// shown second or fraction digits (`0.9999999999` is "0:00" under `h:mm` and "1900-01-01" under
/// `yyyy-mm-dd`; gen-m10, T-1732).
pub(super) struct DisplayClock {
    /// Hours, minutes and seconds of the time of day.
    pub(super) hms: (f64, f64, f64),
    /// Whole days after rounding, so a time just short of midnight rolls into the next date.
    pub(super) days: i64,
    /// What the elapsed tokens count from, rounded like the clock whether or not seconds are
    /// shown: `[h]` on 0.4 s short of 3 h is "3", on 0.6 s short "2" (probes/elapsed-no-seconds-boundary,
    /// T-2385). `None` past 9999-12-31.
    pub(super) elapsed: Option<FractionalSeconds>,
}

impl DisplayClock {
    /// `value` is a non-negative serial. Past 9999-12-31 only a lone elapsed token formats and
    /// nothing is rounded here.
    pub(super) fn new(value: f64, fraction_digits: Option<usize>) -> Self {
        let digits = match fraction_digits {
            Some(digits) => Some(digits),
            None if value < f64::from(MAXIMUM_DATE_SERIAL_NUMBER) + 1.0 => Some(0),
            None => None,
        };
        let rounded = digits.and_then(|digits| FractionalSeconds::new(value, digits));
        let Some(rounded) = rounded else {
            let time_fract = value.fract();
            let hours = (time_fract * 24.0).floor();
            let minutes = ((time_fract * 24.0 - hours) * 60.0).floor();
            let seconds = ((((time_fract * 24.0 - hours) * 60.0) - minutes) * 60.0).round();
            return Self {
                hms: (hours, minutes, seconds),
                days: value as i64,
                elapsed: None,
            };
        };
        Self {
            hms: rounded.clock(),
            days: rounded.days(),
            elapsed: Some(rounded),
        }
    }
}

/// What `General` prints in a date section with a date or clock token: the seconds within the
/// minute, in units of the shown second fraction, of the serial after Excel's half-unit rounding
/// step (`x + 0.5/86400` with no fraction digits, `x + 0.05/86400` with `.0`), with that step's
/// float error left in. `TEXT(5,"d General")` is "5 0.5", `TEXT(45000,"d General")` "15 0.499999803",
/// `TEXT(12345.678,"d General")` "18 19.69999995", `TEXT(45000,"s.0 General")` "0.0 0.499997288"
/// (probes/t3378-general-in-date-section, excel-graph 2026-10-31, T-3378; number-formats.md).
pub(super) fn general_clock_residue(value: f64, fraction_digits: Option<usize>) -> f64 {
    let scale = 10f64.powi(i32::try_from(fraction_digits.unwrap_or(0)).unwrap_or(0));
    let shifted = value + 0.5 / (86400.0 * scale);
    let day_fraction = shifted - shifted.floor();
    (day_fraction * 86400.0 * scale) % (60.0 * scale)
}

/// `x` rounded to `1/scale`; a magnitude whose scaled value overflows is already integral, and
/// Excel does not turn it into 0 (probes/elapsed-huge-h-1900: 2.08E303 days in `[s]` is #VALUE!).
fn snap(x: f64, scale: f64) -> f64 {
    let scaled = x * scale;
    if scaled.is_finite() {
        scaled.round() / scale
    } else {
        x
    }
}

/// Elapsed `[h]`/`[m]`/`[s]` count for a non-negative serial `value` (a negative one is formatted
/// as its magnitude after a leading minus, see the `ParsePart::Date` arm). `[s]` rounds to nearest
/// (1.7 s is "2", 0.9 s of a negative is "-1") and `[h]`/`[m]` truncate (1.7 min is "1", 0.3 h of
/// a negative is "-0"), in both date systems (probes/elapsed-tiny-negatives-1900,
/// elapsed-fractions-1900, T-1338, T-1346). Noise is absorbed first: `[s]` snaps the seconds at
/// 1e-12 (1.5/86400 days is "2" although it is 1.4999999999999998 s, yet 0.49999999999 s stays
/// "0"), `[h]`/`[m]` snap them to the millisecond ((3-1E-7) h is "3"). Those snaps are fitted to
/// the probes, not documented. Excel has no 32-bit limit (T-1362) and no i64 one (T-1375).
/// Up to 9999-12-31 the count comes from `fraction` instead, the serial rounded to the shown
/// second (T-1732, T-2385), so the snaps only serve a lone token past it.
pub(super) fn elapsed_text(
    value: f64,
    padded: bool,
    unit: ElapsedUnit,
    fraction: Option<&FractionalSeconds>,
) -> String {
    let secs = value * 86400.0;
    let n = match (fraction, unit) {
        (Some(fraction), unit) => fraction.elapsed(unit),
        (None, ElapsedUnit::Second) => snap(secs, 1e12).round(),
        // Excel counts minutes and hours from the serial itself once the seconds overflow.
        (None, ElapsedUnit::Minute) if secs.is_infinite() => (value * 1440.0).floor(),
        (None, ElapsedUnit::Hour) if secs.is_infinite() => (value * 24.0).floor(),
        (None, ElapsedUnit::Minute) => (snap(secs, 1e3) / 60.0).floor(),
        (None, ElapsedUnit::Hour) => (snap(secs, 1e3) / 3600.0).floor(),
    };
    // Excel: a count that overflows f64 prints as 0, and from 1e15 up the digits are the first 15
    // significant ones followed by zeros (docs/excel-semantics/number-formats.md, "Elapsed tokens
    // beyond the i64 range"; probes/elapsed-huge-*).
    let (sign, mag) = if n.is_sign_negative() {
        ("-", -n)
    } else {
        ("", n)
    };
    let digits = count_digits(mag);
    if padded {
        format!("{sign}{digits:0>2}")
    } else {
        format!("{sign}{digits}")
    }
}

/// Digits of a non-negative integral count: exact below 1e15, from there the first 15 significant
/// digits then zeros, and "0" for a count that overflowed f64 (probes/elapsed-huge-*).
fn count_digits(mag: f64) -> String {
    if !mag.is_finite() {
        "0".to_owned()
    } else if mag < 1e15 {
        format!("{mag:.0}")
    } else {
        let sci = format!("{mag:.14e}");
        let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
        let exp: usize = exp.parse().unwrap_or(0);
        let mut d: String = mantissa.chars().filter(char::is_ascii_digit).collect();
        d.extend(std::iter::repeat_n('0', exp.saturating_sub(14)));
        d
    }
}

#[cfg(test)]
mod tests {
    use super::{
        elapsed_text,
        ElapsedUnit::{Hour, Minute, Second},
    };

    #[test]
    fn huge_counts_keep_fifteen_significant_digits() {
        assert_eq!(
            elapsed_text(1e15, false, Second, None),
            "86400000000000000000"
        );
        assert_eq!(
            elapsed_text(1e100, false, Hour, None),
            format!("24{}", "0".repeat(100))
        );
        assert_eq!(elapsed_text(1e10, false, Second, None), "864000000000000");
    }

    #[test]
    fn overflowed_counts_are_zero() {
        assert_eq!(elapsed_text(2.0807e303, false, Second, None), "0");
        assert_eq!(elapsed_text(1.5e307, false, Hour, None), "0");
        assert_eq!(elapsed_text(1.5e307, false, Minute, None), "0");
    }

    #[test]
    fn finite_huge_counts_do_not_collapse_to_zero() {
        assert!(elapsed_text(2.08e303, false, Second, None).len() > 300);
        assert!(elapsed_text(1e305, false, Minute, None).len() > 300);
    }
}
