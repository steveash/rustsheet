use crate::{
    arithmetic::flush_subnormal,
    formatter::{
        self,
        format::{FormatContext, Formatted},
        lexer::is_likely_date_number_format,
    },
    locale::get_locale,
    model::{FmtSettings, Model},
    types::NumFmt,
};

// The standard built-in number formats defined by ECMA-376 (§18.8.30), indexed by `numFmtId`
// so that `DEFAULT_NUM_FMTS[id]` is the format code for that built-in id. Ids 23–36 are
// reserved/undefined by the spec and resolve to "general". A workbook-defined `numFmt` always
// takes precedence over this table (see `get_num_fmt`), so this is only consulted for a
// referenced-but-undefined built-in id.
const DEFAULT_NUM_FMTS: &[&str] = &[
    "general",                                                              // 0
    "0",                                                                    // 1
    "0.00",                                                                 // 2
    "#,##0",                                                                // 3
    "#,##0.00",                                                             // 4
    "$#,##0_);($#,##0)",                                                    // 5
    "$#,##0_);[Red]($#,##0)",                                               // 6
    "$#,##0.00_);($#,##0.00)",                                              // 7
    "$#,##0.00_);[Red]($#,##0.00)",                                         // 8
    "0%",                                                                   // 9
    "0.00%",                                                                // 10
    "0.00E+00",                                                             // 11
    "# ?/?",                                                                // 12
    "# ??/??",                                                              // 13
    "mm-dd-yy",                                                             // 14
    "d-mmm-yy",                                                             // 15
    "d-mmm",                                                                // 16
    "mmm-yy",                                                               // 17
    "h:mm AM/PM",                                                           // 18
    "h:mm:ss AM/PM",                                                        // 19
    "h:mm",                                                                 // 20
    "h:mm:ss",                                                              // 21
    "m/d/yy h:mm",                                                          // 22
    "general",                                                              // 23 (reserved)
    "general",                                                              // 24 (reserved)
    "general",                                                              // 25 (reserved)
    "general",                                                              // 26 (reserved)
    "general",                                                              // 27 (reserved)
    "general",                                                              // 28 (reserved)
    "general",                                                              // 29 (reserved)
    "general",                                                              // 30 (reserved)
    "general",                                                              // 31 (reserved)
    "general",                                                              // 32 (reserved)
    "general",                                                              // 33 (reserved)
    "general",                                                              // 34 (reserved)
    "general",                                                              // 35 (reserved)
    "general",                                                              // 36 (reserved)
    "#,##0_);(#,##0)",                                                      // 37
    "#,##0_);[Red](#,##0)",                                                 // 38
    "#,##0.00_);(#,##0.00)",                                                // 39
    "#,##0.00_);[Red](#,##0.00)",                                           // 40
    "_(* #,##0_);_(* \\(#,##0\\);_(* \"-\"_);_(@_)",                        // 41
    "_(\"$\"* #,##0_);_(\"$\"* \\(#,##0\\);_(\"$\"* \"-\"_);_(@_)",         // 42
    "_(* #,##0.00_);_(* \\(#,##0.00\\);_(* \"-\"??_);_(@_)",                // 43
    "_(\"$\"* #,##0.00_);_(\"$\"* \\(#,##0.00\\);_(\"$\"* \"-\"??_);_(@_)", // 44
    "mm:ss",                                                                // 45
    "[h]:mm:ss",                                                            // 46
    "mmss.0",                                                               // 47
    "##0.0E+0",                                                             // 48
    "@",                                                                    // 49
];

pub fn get_default_num_fmt_id(num_fmt: &str) -> Option<i32> {
    for (index, default_num_fmt) in DEFAULT_NUM_FMTS.iter().enumerate() {
        if default_num_fmt == &num_fmt {
            return Some(index as i32);
        };
    }
    None
}

pub fn get_num_fmt(num_fmt_id: i32, num_fmts: &[NumFmt]) -> String {
    // Check if it defined
    for num_fmt in num_fmts {
        if num_fmt.num_fmt_id == num_fmt_id {
            return num_fmt.format_code.clone();
        }
    }
    // Return one of the default ones
    if num_fmt_id < DEFAULT_NUM_FMTS.len() as i32 {
        return DEFAULT_NUM_FMTS[num_fmt_id as usize].to_string();
    }
    // Return general
    DEFAULT_NUM_FMTS[0].to_string()
}

pub fn get_new_num_fmt_index(num_fmts: &[NumFmt]) -> i32 {
    let mut index = DEFAULT_NUM_FMTS.len() as i32;
    let mut found = true;
    while found {
        found = false;
        for num_fmt in num_fmts {
            if num_fmt.num_fmt_id == index {
                found = true;
                index += 1;
                break;
            }
        }
    }
    index
}

/// Parses decimal number text the way Excel reads literals and numeric text: only the first 15
/// significant digits are kept, later digits are truncated (not rounded), so
/// `123456789012345678` reads as `123456789012345000` and `12345.6789012345678` as
/// `12345.6789012345` (docs/excel-semantics/numbers.md "15-digit literal parsing").
/// Accepts what `str::parse::<f64>` accepts, except that the value has to be below 1E+308 in
/// magnitude, Excel's typeable range: "1e308", "1e400", "inf" and "NaN" are `None`
/// (probes/text-number-words A6-A10, H3, K9, K10; docs/excel-semantics/numbers.md
/// "Number text of 1E+308 or more is not a number").
/// At the small end (same section, probes/text-number-tiny, text-number-tiny-zero and
/// literal-number-tiny): a nonzero value below 1E-309 in magnitude is `None`, a value from 1E-309
/// up to the smallest normal double reads as 0, and a zero mantissa is `None` unless its exponent
/// less its fraction digits lies within ±308 (`0e308`, `0.0e309`, `0e-308` are 0; `0e309`,
/// `0e-309`, `0.0e-308` are `None`).
pub fn parse_excel_number(text: &str) -> Option<f64> {
    let (mantissa, exp) = match text.find(['e', 'E']) {
        Some(i) => text.split_at(i),
        None => (text, ""),
    };
    let mut out = String::with_capacity(text.len());
    let mut sig = 0usize;
    let mut in_fraction = false;
    // Integer digits from the first significant one on, and fraction digits read so far: they
    // place the leading digit's power of ten for the small-magnitude rules below.
    let mut integer_sig_digits = 0i64;
    let mut fraction_digits = 0i64;
    let mut leading_fraction_digit = None;
    for c in mantissa.chars() {
        match c {
            '.' => {
                in_fraction = true;
                out.push(c);
            }
            '0'..='9' => {
                if in_fraction {
                    fraction_digits = fraction_digits.saturating_add(1);
                    if c != '0' && sig == 0 {
                        leading_fraction_digit = Some(fraction_digits);
                    }
                } else if sig > 0 || c != '0' {
                    integer_sig_digits = integer_sig_digits.saturating_add(1);
                }
                if sig > 0 || c != '0' {
                    sig += 1;
                }
                if sig <= 15 {
                    out.push(c);
                } else if !in_fraction {
                    out.push('0');
                }
            }
            _ => out.push(c),
        }
    }
    out.push_str(exp);
    let number = out
        .parse::<f64>()
        .ok()
        .filter(|number| in_excel_range(*number))?;
    let exponent = decimal_exponent(exp);
    if sig == 0 {
        // A zero mantissa: Excel tests where its last digit sits, not its value.
        let scale = exponent.saturating_sub(fraction_digits);
        return (scale.abs() <= TINY_EXPONENT_LIMIT).then_some(0.0);
    }
    let leading = match leading_fraction_digit {
        Some(position) => exponent.saturating_sub(position),
        None => exponent.saturating_add(integer_sig_digits - 1),
    };
    (leading >= MIN_LEADING_EXPONENT).then(|| flush_subnormal(number))
}

/// The largest power of ten, either way, a zero mantissa's last digit may sit at: `0e308` and
/// `0e-308` are 0, `0e309` and `0e-309` are not numbers (probes/text-number-tiny-zero).
const TINY_EXPONENT_LIMIT: i64 = 308;

/// The smallest power of ten nonzero number text may start at: `1e-309` reads as 0,
/// `9.9999e-310` and `0.001e-307` are not numbers (probes/text-number-tiny, text-number-tiny-zero).
const MIN_LEADING_EXPONENT: i64 = -309;

/// The exponent of `exp` (`""`, `"e-310"`, `"E+5"`), clamped far beyond any number's range so a
/// long run of digits cannot overflow. `parse_excel_number` has already rejected malformed text.
fn decimal_exponent(exp: &str) -> i64 {
    let digits = exp.trim_start_matches(['e', 'E']);
    let (negative, digits) = match digits.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, digits.strip_prefix('+').unwrap_or(digits)),
    };
    let magnitude = digits
        .bytes()
        .filter(u8::is_ascii_digit)
        .fold(0i64, |acc, d| {
            (acc * 10 + i64::from(d - b'0')).min(1_000_000)
        });
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// Excel's largest typeable number is 9.99999999999999E+307: text or a literal at or above
/// 1E+308 in magnitude is not a number (docs/excel-semantics/numbers.md).
const EXCEL_NUMBER_LIMIT: f64 = 1e308;

/// Whether a parsed number is one Excel reads from text: below [`EXCEL_NUMBER_LIMIT`] in
/// magnitude (false for infinities and NaN too).
fn in_excel_range(number: f64) -> bool {
    number.abs() < EXCEL_NUMBER_LIMIT
}

/// Reads a number written with digits (`12`, `-3.5`, `.5`, `1e5`) the way `str::parse::<f64>`
/// does, minus its surprises: it also reads the words "NaN", "inf" and "infinity", and it reads
/// numbers of 1E+308 or more, which Excel takes for text (`COUNTIF(r,"inf")` counts the text
/// "inf", `COUNT("1e308",1)` is 1, `NUMBERVALUE("1e308")` is #VALUE!: probes/text-number-words
/// E1, L1, L5). Unlike [`parse_excel_number`] it keeps every digit.
pub fn parse_finite_number(text: &str) -> Option<f64> {
    text.parse::<f64>()
        .ok()
        .filter(|number| in_excel_range(*number))
}

pub fn to_precision(value: f64, precision: usize) -> f64 {
    if value.is_infinite() || value.is_nan() {
        return value;
    }
    to_precision_str(value, precision)
        .parse::<f64>()
        .unwrap_or({
            // TODO: do this in a way that does not require a possible error
            0.0
        })
}

/// It rounds a `f64` with `p` significant figures:
/// ```
///     use rustsheet_engine::number_format;
///     assert_eq!(number_format::to_precision(0.1+0.2, 15), 0.3);
///     assert_eq!(number_format::to_excel_precision_str(0.1+0.2), "0.3");
/// ```
/// This intends to be equivalent to the js: `${parseFloat(value.toPrecision(precision)})`
/// See ([ecma](https://tc39.es/ecma262/#sec-number.prototype.toprecision)).
pub fn to_excel_precision_str(value: f64) -> String {
    to_precision_str(value, 15)
}

/// Orders two numbers on their 15-digit values, Excel's comparison rule. There is no absolute
/// tolerance: tiny values never equal 0 (docs/excel-semantics/numbers.md#criteria-equality). NaN
/// is a total order (above every number, equal to itself) so sorts stay consistent.
pub(crate) fn cmp_excel_precision(a: f64, b: f64) -> std::cmp::Ordering {
    // total_cmp orders -0.0 below 0.0 and negative-sign NaN below every number, so canonicalise
    // both first (`+ 0.0` turns -0.0 into 0.0).
    let canon = |x: f64| {
        if x.is_nan() {
            f64::NAN
        } else {
            to_excel_precision(x, 15) + 0.0
        }
    };
    canon(a).total_cmp(&canon(b))
}

pub fn to_excel_precision(value: f64, precision: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let s = format!("{:.*e}", precision.saturating_sub(1), value);
    s.parse::<f64>().unwrap_or(value)
}

/// More significant digits than any double's exact decimal expansion (at most 767).
const EXACT_DIGITS: usize = 800;

/// The exact decimal value of positive finite `value` cut to 15 significant digits:
/// `(mantissa, exponent)` with `value ≈ mantissa / 10^14 * 10^exponent` and
/// `10^14 <= mantissa < 10^15`. Digits past the 15th round to nearest; an exact tie goes away
/// from zero when `tie_away`, otherwise toward zero. Excel's 15-digit cut, shared by the ROUND
/// family (`decimal_round.rs`) and [`number_to_text`] (docs/excel-semantics/numbers.md).
pub(crate) fn significant_15(value: f64, tie_away: bool) -> (u64, i64) {
    // 25 correctly rounded digits decide every case except a possible exact tie, which needs
    // the exact expansion (rare: only values with a 5 as 16th digit and zeros after it).
    let (mut digits, mut exponent) = scientific_digits(value, 25);
    if is_possible_tie(&digits) {
        (digits, exponent) = scientific_digits(value, EXACT_DIGITS);
    }
    let lead = digits.get(..15).unwrap_or_default();
    let mut mantissa = lead
        .iter()
        .fold(0_u64, |acc, d| acc * 10 + u64::from(d - b'0'));
    let rest = digits.get(15..).unwrap_or_default();
    let round_up = match rest.split_first() {
        Some((&first, _)) if first > b'5' => true,
        Some((&b'5', tail)) => tie_away || tail.iter().any(|&d| d != b'0'),
        _ => false,
    };
    if round_up {
        mantissa += 1;
        if mantissa == 1_000_000_000_000_000 {
            mantissa /= 10;
            exponent += 1;
        }
    }
    (mantissa, exponent)
}

/// True when the 16th significant digit is 5 and every later printed digit is 0.
fn is_possible_tie(digits: &[u8]) -> bool {
    digits.get(15) == Some(&b'5') && digits.iter().skip(16).all(|&d| d == b'0')
}

/// The first `count` significant digits of positive finite `value` (correctly rounded) and its
/// decimal exponent.
fn scientific_digits(value: f64, count: usize) -> (Vec<u8>, i64) {
    let text = format!("{:.*e}", count - 1, value);
    let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
    let digits = mantissa.bytes().filter(u8::is_ascii_digit).collect();
    (digits, exponent.parse().unwrap_or(0))
}

/// The text Excel makes of a number when it coerces it to text (`&`, CONCAT, TEXTJOIN, text
/// function arguments). The one routine for every such coercion site.
///
/// Excel (docs/excel-semantics/numbers.md, "Number → text conversion"; probes
/// number-to-text-notation and precision-and-rounding A9, oracle excel-graph): the value is cut
/// to 15 significant digits (exact ties toward zero) and trailing zeros are dropped; it is
/// written in plain notation when that string, without the sign, has at most 20 characters
/// (`99999999999999900000`,
/// `0.000000000000000001`), otherwise as `d.dddE±XX` with at least two exponent digits
/// (`1E+20`, `1.23456789012345E-05`), with 14 digits instead of 15 when the exponent magnitude is
/// 99 or more (`1.2345678901235E+99`). Zero, negative zero and subnormals are `0`.
///
/// Not [`to_excel_precision_str`]: that one renders formula literals (JS `toPrecision`-style,
/// no 20-character rule) and must not be used for coercion.
pub(crate) fn number_to_text(value: f64) -> String {
    /// Longest plain-notation string (sign excluded) before Excel switches to E notation.
    const MAX_PLAIN_LEN: usize = 20;
    if !value.is_finite() {
        // Unreachable from formulas: Excel has no infinities or NaN (overflow is `#NUM!`).
        return format!("{value}");
    }
    if value.abs() < f64::MIN_POSITIVE {
        return "0".to_string();
    }
    let sign = if value < 0.0 { "-" } else { "" };
    // An exact tie at the 16th digit goes toward zero, as in ROUND: `1E+15+15` → "1000000000000010",
    // `1E+14+1.5` → "100000000000001" (probes/number-to-text-notation A45–A52).
    let (mut mantissa, mut exponent) = significant_15(value.abs(), false);
    // From a decimal exponent magnitude of 99 on, Excel keeps 14 digits, rounding the 15-digit
    // string half up (`1.23456789012345E+99` → "1.2345678901235E+99"; probes/number-text-exponent-digits,
    // numbers.md "Exponent width").
    if exponent.unsigned_abs() >= 99 {
        mantissa = (mantissa + 5) / 10 * 10;
        if mantissa == 1_000_000_000_000_000 {
            mantissa /= 10;
            exponent += 1;
        }
    }
    let exponent = i32::try_from(exponent).unwrap_or(0);
    let mantissa = mantissa.to_string();
    let digits = mantissa.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    if plain_len(digits.len(), exponent) <= MAX_PLAIN_LEN {
        return format!("{sign}{}", plain_notation(digits, exponent));
    }
    let (first, rest) = digits.split_at(1);
    let point = if rest.is_empty() { "" } else { "." };
    let exponent_sign = if exponent < 0 { '-' } else { '+' };
    format!(
        "{sign}{first}{point}{rest}E{exponent_sign}{:02}",
        exponent.unsigned_abs()
    )
}

/// Length of [`plain_notation`] without building it (it can be ~300 zeros long).
fn plain_len(count: usize, exponent: i32) -> usize {
    match usize::try_from(exponent) {
        Ok(int_len) if int_len + 1 >= count => int_len + 1,
        Ok(_) => count + 1,
        Err(_) => 1 + exponent.unsigned_abs() as usize + count,
    }
}

/// `digits` (no leading or trailing zeros) times 10^(exponent + 1 - `digits.len()`) in plain
/// decimal notation.
fn plain_notation(digits: &str, exponent: i32) -> String {
    let count = digits.len();
    match usize::try_from(exponent) {
        Ok(int_len) if int_len + 1 >= count => {
            format!("{digits}{}", "0".repeat(int_len + 1 - count))
        }
        Ok(int_len) => {
            let (int_part, frac) = digits.split_at(int_len + 1);
            format!("{int_part}.{frac}")
        }
        Err(_) => {
            let zeros = exponent.unsigned_abs().saturating_sub(1) as usize;
            format!("0.{}{digits}", "0".repeat(zeros))
        }
    }
}

pub fn to_precision_str(value: f64, precision: usize) -> String {
    if !value.is_finite() {
        if value.is_infinite() {
            return "inf".to_string();
        } else {
            return "NaN".to_string();
        }
    }

    let s = format!("{:.*e}", precision.saturating_sub(1), value);
    let parsed = s.parse::<f64>().unwrap_or(value);

    // I would love to use the std library. There is not a speed concern here
    // problem is it doesn't do the right thing
    // Also ryu is my favorite _modern_ algorithm
    let mut buffer = ryu::Buffer::new();
    let text = buffer.format(parsed);
    // The above algorithm converts 2 to 2.0 regrettably
    if let Some(stripped) = text.strip_suffix(".0") {
        return stripped.to_string();
    }
    text.to_string()
}

pub fn format_number(value: f64, format_code: &str, locale: &str) -> Formatted {
    let locale = match get_locale(locale) {
        Ok(l) => l,
        Err(_) => {
            return Formatted {
                text: "#ERROR!".to_owned(),
                color: None,
                error: Some("Invalid locale".to_string()),
            }
        }
    };
    formatter::format::format_number(value, format_code, &FormatContext::new(locale))
}

/// A model's number-formatting inputs and locale examples.
impl Model {
    /// Style of typed input that implies the number format `num_fmt`. The format is not applied
    /// when a date is typed into a cell that is already date-formatted.
    pub(crate) fn style_with_typed_format(
        &mut self,
        style_index: i32,
        num_fmt: &str,
    ) -> Result<i32, String> {
        let keeps_format =
            is_likely_date_number_format(&self.workbook.styles.get_style(style_index)?.num_fmt)
                && is_likely_date_number_format(num_fmt);
        if keeps_format {
            Ok(style_index)
        } else {
            self.workbook
                .styles
                .get_style_with_format(style_index, num_fmt)
        }
    }

    /// The workbook's number-formatting inputs: its locale and date system (SPEC-0005).
    pub(crate) fn format_context(&self) -> FormatContext<'static> {
        FormatContext {
            locale: self.locale,
            date_system: self.workbook.settings.date_system,
        }
    }

    /// Gets the formatting settings based on the locale
    pub fn get_fmt_settings(&self) -> FmtSettings {
        let day_example = 46006.0; // December 15, 2025
        let currency = self.locale.currency.iso.clone();
        let currency_symbol = &self.locale.currency.symbol;
        // "M/d/yy"
        let short_date = &self.locale.dates.date_formats.short;
        // "M/d/yyyy"
        let long_date = &self.locale.dates.date_formats.long;
        // The examples describe the locale, not this workbook: always the 1900 date system.
        let ctx = FormatContext::new(self.locale);
        let short_date_example =
            formatter::format::format_number(day_example, short_date, &ctx).text;
        let long_date_example = formatter::format::format_number(day_example, long_date, &ctx).text;
        // Number format ("#,##0.###")
        // The CLDR formats are a bit different than Excel's
        // let number_fmt = self.locale.numbers.decimal_formats.standard.clone();
        // "#,##0.00 ¤" Currency format might have weird spaces
        let currency_format_template = &self.locale.numbers.currency_formats.standard;
        let currency_format = currency_format_template
            .replace("¤", &format!("\"{currency_symbol}\""))
            .replace(" ", " ");

        let number_fmt = "#,##0.00".to_string();
        let number_example = formatter::format::format_number(1234.567, &number_fmt, &ctx).text;
        FmtSettings {
            currency,
            currency_format,
            short_date: short_date.clone(),
            long_date: long_date.clone(),
            short_date_example,
            long_date_example,
            number_fmt,
            number_example,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The default table is indexed by `numFmtId`: `get_num_fmt(id, &[])` must return the
    // ECMA-376 §18.8.30 built-in code for that id.
    #[test]
    fn builtin_ids_map_to_the_ecma376_codes() {
        assert_eq!(get_num_fmt(0, &[]), "general");
        assert_eq!(get_num_fmt(5, &[]), "$#,##0_);($#,##0)");
        assert_eq!(get_num_fmt(8, &[]), "$#,##0.00_);[Red]($#,##0.00)");
        assert_eq!(get_num_fmt(11, &[]), "0.00E+00");
        assert_eq!(get_num_fmt(37, &[]), "#,##0_);(#,##0)");
        // id 39 is the one that produced `#VALUE!` in the wild (was "t0.00").
        assert_eq!(get_num_fmt(39, &[]), "#,##0.00_);(#,##0.00)");
        assert_eq!(
            get_num_fmt(44, &[]),
            "_(\"$\"* #,##0.00_);_(\"$\"* \\(#,##0.00\\);_(\"$\"* \"-\"??_);_(@_)"
        );
        assert_eq!(get_num_fmt(49, &[]), "@");
        // Reserved ids 23–36 resolve to "general".
        assert_eq!(get_num_fmt(25, &[]), "general");
        // Out-of-range falls back to "general" too.
        assert_eq!(get_num_fmt(500, &[]), "general");
    }

    // A workbook-defined `numFmt` still wins over the default table.
    #[test]
    fn workbook_defined_num_fmt_takes_precedence() {
        let defined = vec![NumFmt {
            num_fmt_id: 39,
            format_code: "FILE-OWN".to_string(),
        }];
        assert_eq!(get_num_fmt(39, &defined), "FILE-OWN");
    }

    // Every corrected built-in code must actually be formattable by IronCalc's own formatter
    // (a valid number must NOT come back as an error). This is what was broken: the old garbage
    // codes (e.g. "t0.00") made the formatter fail on valid numbers.
    //
    // Exception: id 47 (`mmss.0`, elapsed minutes:seconds.tenths) is the correct ECMA-376 code
    // but IronCalc's *formatter* does not yet parse that pattern — a separate formatter gap, not
    // a table bug (the old code at id 47 didn't render either). Kept correct in the table so a
    // future formatter fix makes it work; excluded from this render smoke-check.
    #[test]
    fn corrected_builtins_format_without_error() {
        let mut failures = Vec::new();
        for id in [
            5, 6, 7, 8, 11, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 48, 49,
        ] {
            let code = get_num_fmt(id, &[]);
            let out = format_number(1234.5, &code, "en");
            if out.error.is_some() || out.text.contains("#VALUE!") || out.text.contains("#ERROR!") {
                failures.push((id, code, out.text, out.error));
            }
        }
        assert!(
            failures.is_empty(),
            "codes the formatter rejected: {failures:?}"
        );
    }

    // The specific regression: id 39 formats 175000 as a grouped 2-decimal number, not `#VALUE!`.
    #[test]
    fn id_39_formats_currency_value() {
        let out = format_number(175000.0, &get_num_fmt(39, &[]), "en");
        assert!(out.error.is_none());
        assert!(out.text.contains("175,000.00"), "got {:?}", out.text);
    }
}

#[cfg(test)]
mod number_to_text_tests {
    use super::number_to_text;

    #[test]
    fn matches_the_oracle_probes() {
        let cases: [(f64, &str); 25] = [
            (0.1 + 0.2, "0.3"),
            (1.0 / 3.0, "0.333333333333333"),
            (-2.0 / 3.0, "-0.666666666666667"),
            (2f64.sqrt(), "1.4142135623731"),
            (1e15 + 0.3, "1000000000000000"),
            (123_456_789_012_345_000.0, "123456789012345000"),
            (99_999_999_999_999_900_000.0, "99999999999999900000"),
            (1e20, "1E+20"),
            (-1e19, "-10000000000000000000"),
            (1e-18, "0.000000000000000001"),
            (1e-19, "1E-19"),
            (1.23e-17, "1.23E-17"),
            (1.234_567_890_123_45e-5, "1.23456789012345E-05"),
            (2.220_446_049_250_313e-16, "2.22044604925031E-16"),
            (1.5e300, "1.5E+300"),
            (-0.0, "0"),
            (5e-324, "0"),
            (42.0, "42"),
            (1e15 + 15.0, "1000000000000010"),
            (1e14 + 1.5, "100000000000001"),
            (12_345_678_901_234.75, "12345678901234.7"),
            // T-0199 (evidence T-0186): 14 mantissa digits from a decimal exponent magnitude of 99.
            (1.234_567_890_123_45e98, "1.23456789012345E+98"),
            (1.234_567_890_123_45e99, "1.2345678901235E+99"),
            (9.876_543_210_987_65e-99, "9.8765432109877E-99"),
            (1.234_567_890_123_45e-98, "1.23456789012345E-98"),
        ];
        for (value, text) in cases {
            assert_eq!(number_to_text(value), text, "{value}");
        }
    }
}
