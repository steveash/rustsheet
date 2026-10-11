//! Elapsed `[h]`/`[m]`/`[s]` token text.

/// The unit of an elapsed `[h]`/`[m]`/`[s]` token.
#[derive(Clone, Copy)]
pub(super) enum ElapsedUnit {
    Hour,
    Minute,
    Second,
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
pub(super) fn elapsed_text(value: f64, padded: bool, unit: ElapsedUnit) -> String {
    let secs = value * 86400.0;
    let n = match unit {
        ElapsedUnit::Second => snap(secs, 1e12).round(),
        // Excel counts minutes and hours from the serial itself once the seconds overflow.
        ElapsedUnit::Minute if secs.is_infinite() => (value * 1440.0).floor(),
        ElapsedUnit::Hour if secs.is_infinite() => (value * 24.0).floor(),
        ElapsedUnit::Minute => (snap(secs, 1e3) / 60.0).floor(),
        ElapsedUnit::Hour => (snap(secs, 1e3) / 3600.0).floor(),
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
        assert_eq!(elapsed_text(1e15, false, Second), "86400000000000000000");
        assert_eq!(
            elapsed_text(1e100, false, Hour),
            format!("24{}", "0".repeat(100))
        );
        assert_eq!(elapsed_text(1e10, false, Second), "864000000000000");
    }

    #[test]
    fn overflowed_counts_are_zero() {
        assert_eq!(elapsed_text(2.0807e303, false, Second), "0");
        assert_eq!(elapsed_text(1.5e307, false, Hour), "0");
        assert_eq!(elapsed_text(1.5e307, false, Minute), "0");
    }

    #[test]
    fn finite_huge_counts_do_not_collapse_to_zero() {
        assert!(elapsed_text(2.08e303, false, Second).len() > 300);
        assert!(elapsed_text(1e305, false, Minute).len() > 300);
    }
}
