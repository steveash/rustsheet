//! Excel criteria and wildcard matching: the one place that parses `COUNTIF`-style
//! criteria strings (operators, wildcards, `~` escapes, number/date/bool/error
//! literals) and wildcard text patterns used by MATCH/XMATCH/XLOOKUP and D-functions.

#[cfg(target_arch = "wasm32")]
use regex_lite as regex;

use crate::{
    calc_result::CalcResult,
    expressions::token::{is_english_error_string, Error},
    formatter::date_text::{parse_date, CurrentYear},
    functions::util::collation::cmp_text,
    locale::Locale,
    number_format::{cmp_excel_precision, parse_excel_number, parse_finite_number},
    text::Text,
    types::DateSystem,
};

/// If `s` looks like a date literal in the given locale, return its Excel
/// serial number as `f64`. Pure numeric strings are rejected here because the
/// numeric branch in `build_criteria` handles them already, and date parsing
/// must not shadow the simpler number path.
fn parse_date_criterion(
    s: &str,
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() || parse_finite_number(trimmed).is_some() {
        return None;
    }
    // A 1904 workbook counts the date from 1904-01-01 (probes/date-system-1904-readers A24, A25).
    let (serial, _) = parse_date(trimmed, locale, current_year).ok()?;
    date_system
        .serial_from_1900_serial(i64::from(serial))
        .map(|serial| serial as f64)
}

/// We convert an Excel wildcard into a Rust (Perl family) regex
pub(crate) fn from_wildcard_to_regex(
    wildcard: &str,
    exact: bool,
) -> Result<regex::Regex, regex::Error> {
    // 1. Escape all
    let reg = &regex::escape(wildcard);

    // 2. We convert the escaped '?' into '.' (matches a single character)
    let reg = &reg.replace("\\?", ".");
    // 3. We convert the escaped '*' into '.*' (matches anything)
    let reg = &reg.replace("\\*", ".*");

    // 4. We send '\\~\\~' to '??' that is an unescaped regular expression, therefore cannot be in reg
    let reg = &reg.replace("\\~\\~", "??");

    // 5. If the escaped and converted '*' is preceded by '~' then it's a raw '*'
    let reg = &reg.replace("\\~.*", "\\*");
    // 6. If the escaped and converted '.' is preceded by '~' then it's a raw '?'
    let reg = &reg.replace("\\~.", "\\?");
    // '~' is used in Excel to escape any other character.
    //    So ~x goes to x (whatever x is)
    // 7. Remove all the others '\\~d' --> 'd'
    let reg = &reg.replace("\\~", "");
    // 8. Put back the '\\~\\~'  as '\\~'
    let reg = &reg.replace("??", "\\~");

    // And we have a valid Perl regex! (As Kim Kardashian said before me: "I know, right?")
    if exact {
        return regex::Regex::new(&format!("^{reg}$"));
    }
    regex::Regex::new(reg)
}

// Excel: criteria equality compares the 15-digit values, so 1+2^-52 matches 1 but 1E-17 never matches 0
// (docs/excel-semantics/numbers.md#criteria-equality, probes/tiny-vs-zero-criteria A1-A12).
fn numbers_equal(a: f64, b: f64) -> bool {
    cmp_excel_precision(a, b) == std::cmp::Ordering::Equal
}

// NUMBERS ///
//*********///

// It could be either the number or a string representation of the number
// In the rest of the cases calc_result needs to be a number (cannot be the string "23", for instance)
fn result_is_equal_to_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => numbers_equal(*f, target),
        // Excel: text cells are parsed with the same ASCII-space trimming as the criterion
        // operand (docs/excel-semantics/criteria.md, probes/criteria-padded-text).
        CalcResult::String(s) => {
            if let Some(f) = parse_criterion_number(s) {
                return numbers_equal(f, target);
            }
            false
        }
        _ => false,
    }
}

fn result_is_less_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f < target,
        _ => false,
    }
}

fn result_is_less_or_equal_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f <= target,
        _ => false,
    }
}

fn result_is_greater_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f > target,
        _ => false,
    }
}

fn result_is_greater_or_equal_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f >= target,
        _ => false,
    }
}

fn result_is_not_equal_to_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => !numbers_equal(*f, target),
        _ => true,
    }
}

// BOOLEANS ///
//**********///

// Booleans have to be "exactly" equal
fn result_is_equal_to_bool(calc_result: &CalcResult, target: bool) -> bool {
    match calc_result {
        CalcResult::Boolean(f) => target == *f,
        _ => false,
    }
}

fn result_is_not_equal_to_bool(calc_result: &CalcResult, target: bool) -> bool {
    match calc_result {
        CalcResult::Boolean(f) => target != *f,
        _ => true,
    }
}

// STRINGS ///
//*********///

// Text criteria compare with `cmp_text`, which ignores case and reads a lone surrogate as its own
// unit (docs/excel-semantics/text.md "Lone surrogates", criteria paragraph;
// probes/criteria-lone-surrogates A1-A8, A18-A21).

fn result_is_equal_to_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        // Excel: `COUNTIF(r,"ae")` counts "æ" and `"a-b"` does not equal "ab" (the collation of
        // `=`, docs/excel-semantics/criteria.md "COUNTIF text criteria use the collation").
        CalcResult::String(s) => cmp_text(s, target).is_eq(),
        CalcResult::EmptyCell => target.is_empty(),
        _ => false,
    }
}

// `target` must be lower case.
fn result_starts_with_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => s.to_lowercase().starts_with(target),
        CalcResult::EmptyCell => target.is_empty(),
        _ => false,
    }
}

// `<>text` matches every cell that is not that text, including numbers, booleans,
// errors and blanks (same as the number/bool variants above).
fn result_is_not_equal_to_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        CalcResult::String(s) => cmp_text(s, target).is_ne(),
        _ => true,
    }
}

fn result_is_less_than_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        CalcResult::String(s) => cmp_text(s, target).is_lt(),
        _ => false,
    }
}

// Text `<=` / `>=` criteria compare the cell with the operand (probes/date-text-dotted-criteria
// E10-E13: "10.1.2026" and "10.3.2026" are `<="10.3.2026"`, not `<="1.3.2026"`).
fn result_is_less_or_equal_than_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        CalcResult::String(s) => cmp_text(s, target).is_le(),
        _ => false,
    }
}

fn result_is_greater_than_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        CalcResult::String(s) => cmp_text(s, target).is_gt(),
        _ => false,
    }
}

fn result_is_greater_or_equal_than_string(calc_result: &CalcResult, target: &Text) -> bool {
    match calc_result {
        CalcResult::String(s) => cmp_text(s, target).is_ge(),
        _ => false,
    }
}

// ERRORS ///
//********///

fn result_is_equal_to_error(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::Error { error, .. } => target == error.to_string(),
        _ => false,
    }
}

fn result_is_not_equal_to_error(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::Error { error, .. } => target != error.to_string(),
        _ => true,
    }
}

// EMPTY ///
//*******///

// Note that these two are not inverse of each other.
// In particular, you can never match an empty cell.

fn result_is_not_equal_to_empty(calc_result: &CalcResult) -> bool {
    !matches!(calc_result, CalcResult::EmptyCell)
}

fn result_is_equal_to_empty(calc_result: &CalcResult) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f == 0.0,
        _ => false,
    }
}

/// Which family of functions a criterion is being compiled for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CriteriaFlavor {
    /// COUNTIF/SUMIF/AVERAGEIF and the *IFS variants.
    Standard,
    /// D-functions (DSUM, DCOUNT, ...): identical grammar, except that plain text
    /// equality matches by prefix ("ab" matches "abc").
    // UNVERIFIED: oracle evidence pending, see T-0206.
    Database,
}

/// This returns a function (closure) of signature fn(&CalcResult) -> bool
/// It is Boxed because it returns different closures, so the size cannot be known at compile time
/// The lifetime (a) of value has to be longer or equal to the lifetime of the returned closure
///
/// `locale` is used to recognise date literals in the criterion string (e.g.
/// "<7/31/2023" or "<31/7/2023" depending on the locale's short date format).
/// When the post-operator string is not parseable as a number, we try to parse
/// it as a date and, on success, fall back to numeric comparison against the
/// resulting Excel serial. This is what lets COUNTIF/SUMIF/AVERAGEIF (and the
/// *IFS variants) match date-serial cells with date-string criteria. `current_year` reads
/// year-less dates ("7/1" is 1 July this year; probes/date-text-yearless-3 G1-G4).
pub(crate) fn build_criteria<'a>(
    value: &'a CalcResult,
    locale: &'static Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Box<dyn Fn(&CalcResult) -> bool + 'a> {
    build_criteria_with(
        value,
        locale,
        current_year,
        date_system,
        CriteriaFlavor::Standard,
    )
}

/// A compiled criterion: the predicate a cell value must satisfy.
type CriteriaMatcher<'a> = Box<dyn Fn(&CalcResult) -> bool + 'a>;

/// Matcher for `<`, `>`, `<=`, `>=` followed by a boolean or error operand.
/// Excel: `<TRUE`, `>TRUE`, `<#N/A` and `>#N/A` match nothing, while `<=` and `>=` behave as
/// equality (`COUNTIF` over 10, "abc", TRUE, #N/A, "abd", 5 gives 0 for `>TRUE`/`<TRUE`/`>#N/A`/
/// `<#N/A` and 1 for `>=TRUE`/`<=TRUE`/`>=#N/A`/`<=#N/A`; probes/criteria-operands C1-C8).
fn bool_or_error_operand_matcher<'a>(
    operand: &'a str,
    inclusive: bool,
) -> Option<CriteriaMatcher<'a>> {
    if let Ok(b) = operand.to_lowercase().parse::<bool>() {
        Some(if inclusive {
            Box::new(move |x| result_is_equal_to_bool(x, b))
        } else {
            Box::new(|_| false)
        })
    } else if is_english_error_string(operand) {
        Some(if inclusive {
            Box::new(move |x| result_is_equal_to_error(x, operand))
        } else {
            Box::new(|_| false)
        })
    } else {
        None
    }
}

/// Parses a numeric criterion operand. Excel ignores spaces around it (`"> 1"`,
/// `">1 "`, `"=  1  "` all compare numerically); a tab is not ignored.
/// Evidence: probes/criteria-padded-operand C1-C17 and probes/criteria-gaps C16-C18, C22.
/// "inf", "NaN" and "1e400" are text criteria (probes/text-number-words E1-E5).
fn parse_criterion_number(v: &str) -> Option<f64> {
    parse_excel_number(v.trim_matches(' '))
}

/// Same as [`build_criteria`] with an explicit [`CriteriaFlavor`].
pub(crate) fn build_criteria_with<'a>(
    value: &'a CalcResult,
    locale: &'static Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
    flavor: CriteriaFlavor,
) -> Box<dyn Fn(&CalcResult) -> bool + 'a> {
    match value {
        CalcResult::String(s) => {
            if let Some(v) = s.strip_prefix("<=") {
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_less_or_equal_than_number(x, f))
                } else if let Some(m) = bool_or_error_operand_matcher(v, true) {
                    m
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_less_or_equal_than_number(x, f))
                } else {
                    let target = operand_text(s, "<=");
                    Box::new(move |x| result_is_less_or_equal_than_string(x, &target))
                }
            } else if let Some(v) = s.strip_prefix(">=") {
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_greater_or_equal_than_number(x, f))
                } else if let Some(m) = bool_or_error_operand_matcher(v, true) {
                    m
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_greater_or_equal_than_number(x, f))
                } else {
                    let target = operand_text(s, ">=");
                    Box::new(move |x| result_is_greater_or_equal_than_string(x, &target))
                }
            } else if let Some(v) = s.strip_prefix("<>") {
                // Same for both flavors: `<>ab` is an exact not-equal, never a prefix test.
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_not_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_not_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_not_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    let pattern = WildcardPattern::new(&operand_text(s, "<>"));
                    Box::new(move |x| !pattern.matches_result(x))
                } else if v.is_empty() {
                    Box::new(result_is_not_equal_to_empty)
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_not_equal_to_number(x, f))
                } else {
                    let target = operand_text(s, "<>");
                    Box::new(move |x| result_is_not_equal_to_string(x, &target))
                }
            } else if let Some(v) = s.strip_prefix('<') {
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_less_than_number(x, f))
                } else if let Some(m) = bool_or_error_operand_matcher(v, false) {
                    m
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_less_than_number(x, f))
                } else {
                    let target = operand_text(s, "<");
                    Box::new(move |x| result_is_less_than_string(x, &target))
                }
            } else if let Some(v) = s.strip_prefix('>') {
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_greater_than_number(x, f))
                } else if let Some(m) = bool_or_error_operand_matcher(v, false) {
                    m
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_greater_than_number(x, f))
                } else {
                    let target = operand_text(s, ">");
                    Box::new(move |x| result_is_greater_than_string(x, &target))
                }
            } else {
                let explicit_equals = s.starts_with('=');
                let v = s.strip_prefix('=').unwrap_or(s);
                if let Some(f) = parse_criterion_number(v) {
                    Box::new(move |x| result_is_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    let pattern = WildcardPattern::new(&operand_text(
                        s,
                        if explicit_equals { "=" } else { "" },
                    ));
                    Box::new(move |x| pattern.matches_result(x))
                } else if let Some(f) = parse_date_criterion(v, locale, current_year, date_system) {
                    Box::new(move |x| result_is_equal_to_number(x, f))
                } else if flavor == CriteriaFlavor::Standard && explicit_equals && v.is_empty() {
                    // Excel: a bare "=" does not match an empty-string cell (probes/criteria-grammar
                    // Sheet1!C4); a blank cell still matches (probes/criteria-gaps C2).
                    Box::new(move |x| matches!(x, CalcResult::EmptyCell))
                } else if flavor == CriteriaFlavor::Database && !explicit_equals {
                    Box::new(move |x| result_starts_with_string(x, &v.to_lowercase()))
                } else {
                    let target = operand_text(s, if explicit_equals { "=" } else { "" });
                    Box::new(move |x| result_is_equal_to_string(x, &target))
                }
            }
        }
        CalcResult::Number(target) => Box::new(move |x| result_is_equal_to_number(x, *target)),
        CalcResult::Boolean(b) => Box::new(move |x| result_is_equal_to_bool(x, *b)),
        CalcResult::Error { error, .. } => {
            // An error will match an error (never a string that is an error). Excel: except a stored
            // #BUSY!, which matches nothing, not even its own cell (probes/error-stored-busy-behaviors
            // C15; a #N/A criterion does match #N/A cells).
            if *error == Error::BUSY {
                return Box::new(|_| false);
            }
            Box::new(move |x| result_is_equal_to_error(x, &error.to_string()))
        }
        CalcResult::Range { left: _, right: _ } => Box::new(move |_x| false),
        CalcResult::Array(_) | CalcResult::Lambda(_) => Box::new(move |_x| false),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Box::new(result_is_equal_to_empty),
    }
}

/// Matcher for an Excel wildcard text pattern (`*`, `?`, `~` escapes), as used by
/// MATCH/XMATCH/XLOOKUP in wildcard mode: [`WildcardPattern`] over text values; anything that is
/// not text never matches.
pub(crate) fn wildcard_text_matcher(pattern: &Text) -> Box<dyn Fn(&CalcResult) -> bool> {
    let pattern = WildcardPattern::new(pattern);
    Box::new(move |x| pattern.matches_result(x))
}

/// The criterion text after its operator `prefix`, lone surrogates kept. The operators are
/// ASCII, so the prefix's byte length is its length in UTF-16 units.
fn operand_text(criterion: &Text, prefix: &str) -> Text {
    criterion.slice_utf16(prefix.len(), criterion.len_utf16())
}

/// One element of a [`WildcardPattern`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WildcardToken {
    /// A literal UTF-16 unit, lower-cased.
    Unit(u16),
    /// `?`: any one UTF-16 unit.
    AnyUnit,
    /// `*`: any run of units, the empty run included.
    AnyRun,
}

const TILDE: u16 = b'~' as u16;
const STAR: u16 = b'*' as u16;
const QUESTION: u16 = b'?' as u16;

/// An Excel wildcard pattern, matched against whole texts, case-insensitively, UTF-16 unit by
/// unit. Excel: `?` is one unit, so a surrogate pair takes `??` and a wildcard splits a pair
/// (`H&"*"` matches "😀"); a lone surrogate is a unit of its own, never U+FFFD
/// (docs/excel-semantics/text.md "Lone surrogates", criteria paragraph;
/// probes/criteria-lone-surrogates A9-A16, A25-A28, A33-A43). `?` and `*` match line breaks too
/// (probes/criteria-wildcard-newline). `~` makes the next unit literal (`~*`, `~?`, `~~`, and
/// `~x` reads as `x`) and a trailing `~` is dropped, unlike a criterion without wildcards
/// (docs/excel-semantics/criteria.md "Tildes inside wildcard patterns and wildcard-mode lookups";
/// probes/criteria-wildcard-tilde).
pub(crate) struct WildcardPattern {
    tokens: Vec<WildcardToken>,
}

impl WildcardPattern {
    /// Compiles `pattern`.
    pub(crate) fn new(pattern: &Text) -> WildcardPattern {
        let lowered = pattern.map_runs(str::to_lowercase);
        let mut units = lowered.units();
        let mut tokens = Vec::new();
        while let Some(unit) = units.next() {
            let token = match unit {
                STAR => WildcardToken::AnyRun,
                QUESTION => WildcardToken::AnyUnit,
                TILDE => match units.next() {
                    Some(next) => WildcardToken::Unit(next),
                    None => break,
                },
                _ => WildcardToken::Unit(unit),
            };
            tokens.push(token);
        }
        WildcardPattern { tokens }
    }

    /// Whether `value` is text the pattern matches.
    pub(crate) fn matches_result(&self, value: &CalcResult) -> bool {
        matches!(value, CalcResult::String(s) if self.matches(s))
    }

    /// Whether the whole of `text` matches, ignoring case.
    pub(crate) fn matches(&self, text: &Text) -> bool {
        match text {
            // The hot path: ASCII text is its own units, lower-cased byte by byte without a copy.
            Text::Utf8(s) if s.is_ascii() => {
                self.matches_units(s.as_bytes(), |byte| u16::from(byte.to_ascii_lowercase()))
            }
            _ => {
                let units: Vec<u16> = text.map_runs(str::to_lowercase).units().collect();
                self.matches_units(&units, |unit| unit)
            }
        }
    }

    /// Glob matching with backtracking to the last `*` only, which suffices because `*` matches
    /// any run: on a mismatch the last `*` takes one more unit. At most `text.len()` restarts,
    /// each of at most `tokens.len()` steps.
    fn matches_units<T: Copy>(&self, text: &[T], unit: impl Fn(T) -> u16) -> bool {
        let tokens = &self.tokens;
        let mut next_token = 0;
        let mut next_unit = 0;
        // The token after the last `*` seen, and the text position that `*` now runs to.
        let mut last_star: Option<(usize, usize)> = None;
        while let Some(&current) = text.get(next_unit) {
            match tokens.get(next_token) {
                Some(WildcardToken::AnyRun) => {
                    next_token += 1;
                    last_star = Some((next_token, next_unit));
                    continue;
                }
                Some(WildcardToken::AnyUnit) => {
                    next_token += 1;
                    next_unit += 1;
                    continue;
                }
                Some(WildcardToken::Unit(literal)) if *literal == unit(current) => {
                    next_token += 1;
                    next_unit += 1;
                    continue;
                }
                _ => {}
            }
            let Some((after_star, star_end)) = last_star else {
                return false;
            };
            next_token = after_star;
            next_unit = star_end + 1;
            last_star = Some((after_star, next_unit));
        }
        tokens
            .get(next_token..)
            .is_some_and(|rest| rest.iter().all(|token| *token == WildcardToken::AnyRun))
    }
}
