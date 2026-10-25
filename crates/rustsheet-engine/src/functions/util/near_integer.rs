//! Whole numbers read from fractional arguments the way Excel reads them: floored, but a value
//! just below an integer counts as that integer.

/// How far below the next integer a whole-number argument may fall and still count as that
/// integer: 2^-22 + 2^-33, the same at 1, 500, 1000, 30000 and 100000 (an absolute bound, not
/// relative). LEFT, RIGHT, REPLACE counts, INDEX's and VLOOKUP/HLOOKUP's indexes, DATE, TIME and the
/// XMATCH and XLOOKUP modes share it.
pub(crate) const NEAR_INTEGER_TOLERANCE: f64 = 1.0 / 4_194_304.0 + 1.0 / 8_589_934_592.0;

/// The whole number LEFT, RIGHT, REPLACE, INDEX, VLOOKUP and HLOOKUP read from a count, position
/// or index: `x` floored, except that a fraction within [`NEAR_INTEGER_TOLERANCE`] of the next
/// integer counts as that integer, so `LEFT("abc",1-2^-22)` is "a". MID truncates exactly and does
/// not use this.
// Excel: docs/excel-semantics/text.md "LEFT, RIGHT and REPLACE round counts up near an
// integer" (probes/text-count-argument-bounds) and lookup.md "INDEX rounds an index up near an
// integer" (probes/index-snap-tolerance).
pub(crate) fn near_integer_count(x: f64) -> f64 {
    near_integer_floor(x, NEAR_INTEGER_TOLERANCE)
}

/// `x` floored, but a fraction within `tolerance` of the next integer counts as that integer.
pub(crate) fn near_integer_floor(x: f64, tolerance: f64) -> f64 {
    let whole = x.floor();
    // `x - whole` is exact for every finite double, so the bound is not blurred at large x.
    whole + f64::from(u8::from(x - whole >= 1.0 - tolerance))
}

/// The whole number DATE, TIME, XMATCH and XLOOKUP read from an argument: [`near_integer_floor`]
/// with [`NEAR_INTEGER_TOLERANCE`], half of it for a negative value, so `TIME(1-2^-22,0,0)` is
/// 1:00, `TIME(1,0,-1E-7)` 1:00, `DATE(1983,12,-1E-15)` day 0 and an XMATCH mode of
/// `0.5^1E-15` reads 1, while `-0.4` reads -1.
// Excel: docs/excel-semantics/dates.md "TIME: an argument past 32767 is `#NUM!`" and "DATE: a
// year, month or day just below an integer" (probes/time-argument-*, date-argument-window; T-2267,
// T-2268); lookup.md "XMATCH and XLOOKUP floor a fractional mode, snapping near an integer"
// (probes/t2656-xmatch-mode-*, t2656-xlookup-mode-snap-edge; T-2656). Those probes bracket both
// edges to a relative 2^-14, and the constants sit on the snapping side:
// 1 - 2^-22*(1+2^-11) reads 1 and 1 - 2^-22*(1+9*2^-14) reads 0; -2^-23*(1+2^-11) reads 0 and
// -2^-23*(1+9*2^-14) reads -1, the same below -2 through 3.
pub(crate) fn near_integer_whole(value: f64) -> f64 {
    let tolerance = if value < 0.0 {
        NEAR_INTEGER_TOLERANCE / 2.0
    } else {
        NEAR_INTEGER_TOLERANCE
    };
    near_integer_floor(value, tolerance)
}

#[cfg(test)]
mod tests {
    use super::near_integer_count;

    #[test]
    fn near_integer_count_rounds_up_within_the_tolerance() {
        // probes/text-count-argument-bounds Z10, Z9, AA1, AA2: 1 - (2^-22 + 2^-33) is 1, a hair
        // further down is 0; the bound is absolute (AA13, AA14 below 30000).
        let at = 2f64.powi(-22) + 2f64.powi(-33);
        let past = 2f64.powi(-22) + 9.0 * 2f64.powi(-36);
        assert_eq!(near_integer_count(1.0 - at), 1.0);
        assert_eq!(near_integer_count(1.0 - past), 0.0);
        assert_eq!(near_integer_count(30000.0 - at), 30000.0);
        assert_eq!(near_integer_count(30000.0 - past), 29999.0);
        assert_eq!(near_integer_count(2.5), 2.0);
        assert_eq!(near_integer_count(1e-300), 0.0);
        assert_eq!(near_integer_count(1e100), 1e100);
    }
}
