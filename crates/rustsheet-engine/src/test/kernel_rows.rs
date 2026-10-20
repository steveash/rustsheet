#![allow(clippy::unwrap_used)]

//! Excel kernel rows for the T-1941 / T-2123 x87 kernel tests: committed fixtures (a sample of the
//! oracle runs with the research model's value beside Excel's) and, when the research data is on
//! disk, the full runs.
//!
//! A fixture line is `x excel model` as three 16-digit hex `f64` bit patterns; `#` starts a
//! comment. The full runs are `x_hex y_hex case` lines in Python `float.hex` form, as
//! `docs/research/t1941/extract_runs.py` writes them, one file per function (`ln.rows`, ...) in the
//! directory `RUSTSHEET_T1941_ROWS` names.

/// One fixture point: the argument, Excel's value and the research model's value.
pub(crate) struct KernelRow {
    pub x: f64,
    pub excel: f64,
    pub model: f64,
}

/// The rows of a fixture file's text (module docs); a malformed line is a test failure.
pub(crate) fn fixture_rows(text: &str) -> Vec<KernelRow> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let bits: Vec<f64> = line
                .split_whitespace()
                .map(|field| f64::from_bits(u64::from_str_radix(field, 16).unwrap()))
                .collect();
            assert_eq!(bits.len(), 3, "malformed fixture line {line:?}");
            KernelRow {
                x: bits[0],
                excel: bits[1],
                model: bits[2],
            }
        })
        .collect()
}

/// The full runs for `function` (`ln`, `gl` or `ga`) as (x, Excel) pairs, or `None` when
/// `RUSTSHEET_T1941_ROWS` is unset.
pub(crate) fn full_rows(function: &str) -> Option<Vec<(f64, f64)>> {
    let directory = std::env::var("RUSTSHEET_T1941_ROWS").ok()?;
    let text = std::fs::read_to_string(format!("{directory}/{function}.rows")).unwrap();
    Some(
        text.lines()
            .map(|line| {
                let mut fields = line.split_whitespace();
                let x = parse_hex_float(fields.next().unwrap());
                let y = parse_hex_float(fields.next().unwrap());
                (x, y)
            })
            .collect(),
    )
}

/// A Python `float.hex` string (`-0x1.8p+3`, `0x0.0p+0`, subnormals as `0x0.xxxp-1022`).
fn parse_hex_float(text: &str) -> f64 {
    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let rest = rest.strip_prefix("0x").unwrap();
    let (mantissa, exponent) = rest.split_once('p').unwrap();
    let (lead, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let fraction_bits = u64::from_str_radix(&format!("{fraction:0<13}"), 16).unwrap();
    let lead = u64::from_str_radix(lead, 16).unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    // `float.hex` writes a normal number as 0x1.f..p<e> and a zero or subnormal as 0x0.f..p-1022.
    let bits = if lead == 0 {
        fraction_bits
    } else {
        (u64::try_from(exponent + 1023).unwrap() << 52) | fraction_bits
    };
    let value = f64::from_bits(bits);
    if negative {
        -value
    } else {
        value
    }
}

/// How many of `rows` `kernel` gives Excel's exact bits for.
pub(crate) fn exact_count(rows: &[(f64, f64)], kernel: impl Fn(f64) -> f64) -> usize {
    rows.iter()
        .filter(|&&(x, excel)| kernel(x).to_bits() == excel.to_bits())
        .count()
}

#[test]
fn hex_floats_parse() {
    assert_eq!(parse_hex_float("0x1.8000000000000p+3"), 12.0);
    assert_eq!(parse_hex_float("-0x1.0000000000000p-1"), -0.5);
    assert_eq!(parse_hex_float("0x0.0p+0"), 0.0);
    assert_eq!(parse_hex_float("0x1.fffffffffffffp+1023"), f64::MAX);
    assert_eq!(
        parse_hex_float("0x0.0000000000001p-1022"),
        f64::from_bits(1)
    );
    assert_eq!(
        parse_hex_float("0x1.3988e1409212ep-1"),
        f64::from_bits(0x3FE3_988E_1409_212E)
    );
}
