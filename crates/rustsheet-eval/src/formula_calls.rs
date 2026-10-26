//! Which functions a formula calls, as the report attributes cells to functions: every call
//! (`functions_in`) or only the outer one (`outer_call`, decision 0094).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::catalog::Catalog;

/// A call's name, without its `_xlfn.`/`_xlws.` prefix (group 1), up to its open parenthesis.
const CALL: &str = r"(?:_xlfn\.|_xlws\.)?\b([A-Z][A-Z0-9]*(?:\.[A-Z0-9]+)*)\s*\(";

static FUNCTION_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!("(?i){CALL}")).unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static OUTER_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?i)^=?\s*{CALL}")).unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static QUOTED_SHEET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"'(?:[^']|'')*'").unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static STRING_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"]|"")*""#).unwrap_or_else(|e| unreachable!("static regex: {e}"))
});

/// An add-in function call (`_xll.` prefix), case-insensitive.
static ADD_IN_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b_xll\.").unwrap_or_else(|e| unreachable!("static regex: {e}"))
});

/// Whether a formula calls an add-in function (`_xll.`), outside string literals. Decision 0027
/// puts add-in functions out of scope: no engine can compute them (Excel's cached value came from
/// the add-in), so such a cell is not scored (T-0462). Only the calling cell is left out; its
/// dependents are scored, so a miss they show stays visible.
#[must_use]
pub fn calls_add_in(formula: &str) -> bool {
    ADD_IN_CALL.is_match(&STRING_LITERAL.replace_all(formula, "\"\""))
}

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
                    let name = call[1].to_ascii_uppercase();
                    let whole = rest.trim().is_empty()
                        || (name == "LAMBDA" && is_invocation_list(rest.trim()));
                    return whole.then_some(name);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether `text` is one or more `(...)` groups and nothing else: the arguments a LAMBDA is
/// invoked with (`(1)(2)`). A generated LAMBDA cell is always invoked, since a bare one is
/// `#CALC!` (T-2677).
fn is_invocation_list(text: &str) -> bool {
    let mut depth = 0usize;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(inner) = depth.checked_sub(1) else {
                    return false;
                };
                depth = inner;
            }
            _ if depth == 0 && !ch.is_whitespace() => return false,
            _ => {}
        }
        if depth == 0 && i + ch.len_utf8() == text.len() {
            return ch == ')';
        }
    }
    false
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
    fn add_in_calls_are_found_outside_string_literals() {
        // calipers pv26_003 Model!C875 (T-0462).
        assert!(calls_add_in(
            r#"=IFERROR(_xll.ciqfunctions.udf.CIQHI(MO.Ticker.CapIQ,"IQ_LASTSALEPRICE"),"N/A")"#
        ));
        assert!(calls_add_in("=_XLL.FOO(1)+1"));
        assert!(!calls_add_in(r#"=CONCAT("_xll.FOO(",A1)"#));
        assert!(!calls_add_in("=SUM(my_xll.range)"));
        assert!(!calls_add_in("=_xlfn.XLOOKUP(1,A:A,B:B)"));
    }

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

    #[test]
    fn an_invoked_lambda_is_a_lambda_outer_call() {
        // Decision 0094; a bare LAMBDA is `#CALC!`, so generated cells invoke it (T-2677).
        for lambda in [
            "LAMBDA(x,x+1)(3)",
            "=_xlfn.LAMBDA(_xlpm.x,_xlpm.x)(1)",
            "LAMBDA(x,x)(1)(2)",
            "LAMBDA(x,x)(SUM(1,2)) ",
        ] {
            assert_eq!(outer_call(lambda).as_deref(), Some("LAMBDA"), "{lambda}");
        }
        for other in [
            "LAMBDA(x,x)(1)+1",
            "-LAMBDA(x,x)(1)",
            "LAMBDA(x,x)(1",
            "LAMBDA(x,x)(1))",
            "LAMBDA(x,x)(1)A1",
            "ABS(1)(2)",
        ] {
            assert_eq!(outer_call(other), None, "{other}");
        }
    }
}
