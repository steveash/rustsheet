//! Which functions a formula calls, as the report attributes cells to functions: every call
//! (`functions_in`) or only the outer one (`outer_call`, decision 0094).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::catalog::Catalog;

static FUNCTION_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:_xlfn\.|_xlws\.)?\b([A-Z][A-Z0-9]*(?:\.[A-Z0-9]+)*)\s*\(")
        .unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static OUTER_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^=?\s*(?:_xlfn\.|_xlws\.)?([A-Z][A-Z0-9]*(?:\.[A-Z0-9]+)*)\s*\(")
        .unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static QUOTED_SHEET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"'(?:[^']|'')*'").unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static STRING_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"]|"")*""#).unwrap_or_else(|e| unreachable!("static regex: {e}"))
});

/// Upper-cased function names called by a formula (string literals ignored).
#[must_use]
pub fn functions_in(formula: &str) -> BTreeSet<String> {
    let stripped = STRING_LITERAL.replace_all(formula, "\"\"");
    FUNCTION_CALL
        .captures_iter(&stripped)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_ascii_uppercase()))
        .collect()
}

/// The upper-cased function a formula is one call of (`ABS(SQRT(4))` is ABS), or `None` when the
/// formula is anything else at the top (`ABS(1)+SQRT(4)`, `-ABS(1)`, a reference). Decision 0094
/// counts a generated cell for its outer call only.
#[must_use]
pub fn outer_call(formula: &str) -> Option<String> {
    let stripped = STRING_LITERAL.replace_all(formula.trim(), "\"\"");
    let stripped = QUOTED_SHEET.replace_all(&stripped, "''");
    let call = OUTER_CALL.captures(&stripped)?;
    let open = call.get(0)?.end();
    let mut depth = 1usize;
    for (i, ch) in stripped[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let rest = &stripped[open + i + 1..];
                    return rest.trim().is_empty().then(|| call[1].to_ascii_uppercase());
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether a cell calling `called` counts toward parity: it calls no out-of-scope catalog function
/// (decision 0027).
#[must_use]
pub fn calls_only_in_scope(catalog: &Catalog, called: &BTreeSet<String>) -> bool {
    called.iter().all(|f| catalog.in_scope(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_function_names() {
        let f =
            functions_in(r#"IF(_xlfn.XLOOKUP(A1,B:B,C:C)>0,SUM(A1:A3),"NOW()")+ERF.PRECISE(1)"#);
        let v: Vec<&str> = f.iter().map(String::as_str).collect();
        assert_eq!(v, vec!["ERF.PRECISE", "IF", "SUM", "XLOOKUP"]);
    }

    #[test]
    fn outer_call_is_the_call_spanning_the_whole_formula() {
        assert_eq!(outer_call("=_xlfn.ABS(SQRT(4))").as_deref(), Some("ABS"));
        assert_eq!(outer_call(r#"LEN(")+(")"#).as_deref(), Some("LEN"));
        assert_eq!(outer_call("SUM('a (b)'!A1:A2)").as_deref(), Some("SUM"));
        for other in ["ABS(1)+SQRT(4)", "-ABS(1)", "A1", "ABS(1", "(ABS(1))"] {
            assert_eq!(outer_call(other), None, "{other}");
        }
    }
}
