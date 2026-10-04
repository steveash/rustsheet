//! Excel criteria and wildcard matching: the one place that parses `COUNTIF`-style
//! criteria strings (operators, wildcards, `~` escapes, number/date/bool/error
//! literals) and wildcard text patterns used by MATCH/XMATCH/XLOOKUP and D-functions.

#[cfg(target_arch = "wasm32")]
use regex_lite as regex;

use crate::{
    calc_result::CalcResult, expressions::token::is_english_error_string,
    formatter::format::parse_date, locale::Locale,
};

/// If `s` looks like a date literal in the given locale, return its Excel
/// serial number as `f64`. Pure numeric strings are rejected here because the
/// numeric branch in `build_criteria` handles them already, and date parsing
/// must not shadow the simpler number path.
fn parse_date_criterion(s: &str, locale: &Locale) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed.parse::<f64>().is_ok() {
        return None;
    }
    parse_date(trimmed, locale)
        .ok()
        .map(|(serial, _)| serial as f64)
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

// NUMBERS ///
//*********///

// It could be either the number or a string representation of the number
// In the rest of the cases calc_result needs to be a number (cannot be the string "23", for instance)
fn result_is_equal_to_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => {
            if (f - target).abs() < f64::EPSILON {
                return true;
            }
            false
        }
        CalcResult::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                if (f - target).abs() < f64::EPSILON {
                    return true;
                }
                return false;
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
        CalcResult::Number(f) => {
            if (f - target).abs() > f64::EPSILON {
                return true;
            }
            false
        }
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

// Note that strings are case insensitive. `target` must always be lower case.

pub(crate) fn result_matches_regex(calc_result: &CalcResult, reg: &regex::Regex) -> bool {
    match calc_result {
        CalcResult::String(s) => reg.is_match(&s.to_lowercase()),
        _ => false,
    }
}

fn result_is_equal_to_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            if target == s.to_lowercase() {
                return true;
            }
            false
        }
        CalcResult::EmptyCell => target.is_empty(),
        _ => false,
    }
}

fn result_starts_with_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => s.to_lowercase().starts_with(target),
        CalcResult::EmptyCell => target.is_empty(),
        _ => false,
    }
}

// `<>text` matches every cell that is not that text, including numbers, booleans,
// errors and blanks (same as the number/bool variants above).
fn result_is_not_equal_to_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => target != s.to_lowercase(),
        _ => true,
    }
}

fn result_is_less_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => target.cmp(&s.to_lowercase()) == std::cmp::Ordering::Greater,
        _ => false,
    }
}

fn result_is_less_or_equal_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            let lower_case = &s.to_lowercase();
            target.cmp(lower_case) == std::cmp::Ordering::Less || lower_case == target
        }
        _ => false,
    }
}

fn result_is_greater_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => target.cmp(&s.to_lowercase()) == std::cmp::Ordering::Less,
        _ => false,
    }
}

fn result_is_greater_or_equal_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            let lower_case = &s.to_lowercase();
            target.cmp(lower_case) == std::cmp::Ordering::Greater || lower_case == target
        }
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
        CalcResult::Number(f) => (f - 0.0).abs() < f64::EPSILON,
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
/// *IFS variants) match date-serial cells with date-string criteria.
pub(crate) fn build_criteria<'a>(
    value: &'a CalcResult,
    locale: &'static Locale,
) -> Box<dyn Fn(&CalcResult) -> bool + 'a> {
    build_criteria_with(value, locale, CriteriaFlavor::Standard)
}

/// Same as [`build_criteria`] with an explicit [`CriteriaFlavor`].
pub(crate) fn build_criteria_with<'a>(
    value: &'a CalcResult,
    locale: &'static Locale,
    flavor: CriteriaFlavor,
) -> Box<dyn Fn(&CalcResult) -> bool + 'a> {
    match value {
        CalcResult::String(s) => {
            if let Some(v) = s.strip_prefix("<=") {
                // TODO(T-0206): not implementing <= ERROR or <= BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_less_or_equal_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_less_or_equal_than_number(x, f))
                } else {
                    Box::new(move |x| result_is_less_or_equal_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix(">=") {
                // TODO(T-0206): not implementing >= ERROR or >= BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_greater_or_equal_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_greater_or_equal_than_number(x, f))
                } else {
                    Box::new(move |x| result_is_greater_or_equal_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix("<>") {
                // Same for both flavors: `<>ab` is an exact not-equal, never a prefix test.
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_not_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_not_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_not_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    if let Ok(reg) = from_wildcard_to_regex(&v.to_lowercase(), true) {
                        Box::new(move |x| !result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else if v.is_empty() {
                    Box::new(result_is_not_equal_to_empty)
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_not_equal_to_number(x, f))
                } else {
                    Box::new(move |x| result_is_not_equal_to_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix('<') {
                // TODO(T-0206): not implementing < ERROR or < BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_less_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_less_than_number(x, f))
                } else {
                    Box::new(move |x| result_is_less_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix('>') {
                // TODO(T-0206): not implementing > ERROR or > BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_greater_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_greater_than_number(x, f))
                } else {
                    Box::new(move |x| result_is_greater_than_string(x, &v.to_lowercase()))
                }
            } else {
                let v = if let Some(a) = s.strip_prefix('=') {
                    a
                } else {
                    s
                };
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    if let Ok(reg) = from_wildcard_to_regex(&v.to_lowercase(), true) {
                        Box::new(move |x| result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else if let Some(f) = parse_date_criterion(v, locale) {
                    Box::new(move |x| result_is_equal_to_number(x, f))
                } else if flavor == CriteriaFlavor::Database {
                    Box::new(move |x| result_starts_with_string(x, &v.to_lowercase()))
                } else {
                    Box::new(move |x| result_is_equal_to_string(x, &v.to_lowercase()))
                }
            }
        }
        CalcResult::Number(target) => Box::new(move |x| result_is_equal_to_number(x, *target)),
        CalcResult::Boolean(b) => Box::new(move |x| result_is_equal_to_bool(x, *b)),
        CalcResult::Error { error, .. } => {
            // An error will match an error (never a string that is an error)
            Box::new(move |x| result_is_equal_to_error(x, &error.to_string()))
        }
        CalcResult::Range { left: _, right: _ } => Box::new(move |_x| false),
        CalcResult::Array(_) | CalcResult::Lambda(_) => Box::new(move |_x| false),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Box::new(result_is_equal_to_empty),
    }
}

/// Matcher for an Excel wildcard text pattern (`*`, `?`, `~` escapes), as used by
/// MATCH/XMATCH/XLOOKUP in wildcard mode. Matches whole text values
/// case-insensitively; anything that is not text never matches. An
/// untranslatable pattern matches nothing.
pub(crate) fn wildcard_text_matcher(pattern: &str) -> Box<dyn Fn(&CalcResult) -> bool> {
    match from_wildcard_to_regex(&pattern.to_lowercase(), true) {
        Ok(reg) => Box::new(move |x| result_matches_regex(x, &reg)),
        Err(_) => Box::new(|_| false),
    }
}
