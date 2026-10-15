#![allow(clippy::unwrap_used)]

use crate::calc_result::CalcResult;
use crate::functions::criteria::{
    build_criteria, build_criteria_with, wildcard_text_matcher, CriteriaFlavor,
};
use crate::locale::{get_locale, Locale};
use crate::types::DateSystem;

fn en_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

/// The clock year-less date criteria ("7/1") are read with.
fn year_2026() -> Option<i32> {
    Some(2026)
}

// Tests for build_criteria
// ------------------------
//
// Note that any test here is mostly for documentation purposes.
// A real test must be done in Excel.
//
// `build_criteria` takes a string ('criteria') and returns a function ('fn_criteria') that takes a CalcResult and returns a boolean.
//
// For instance if criteria is "123" we want all cells that contain the number "123". Then
//
// let fn_criteria = build_criteria(&CalcResult::Number(123));
//
// Then fn_criteria(calc_result) will return true every time calc_result is the number "123"
//
// There are different types of criteria
//
// * We want the cells that are equal to a value (say a number, string, bool or an error).
//   We can build those with a calc_result of the type (i.e CalcResult::Number(123))
//   or we can use a string preceded by "=" like CalcResult::String("=123")
// * We can use inequality signs "<", ">", "<=", ">=" or "<>"
// * If you use "=" or "<>" you can use wildcards (like "=*brown")
//
// All of them are case insensitive.

#[test]
fn test_build_criteria_is_number() {
    let c = CalcResult::Number(42.0);
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String(("42".to_string()).into())));
    assert!(fn_criteria(&CalcResult::String(
        ("42.00".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));

    let c = CalcResult::String(("=42".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String(("42".to_string()).into())));
    assert!(fn_criteria(&CalcResult::String(
        ("42.00".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));
}

#[test]
fn test_build_criteria_is_bool() {
    let c = CalcResult::Boolean(true);
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String(
        ("true".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));

    let c = CalcResult::String(("=True".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String(
        ("true".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));
}

#[test]
fn test_build_criteria_ordering_with_bool_operand() {
    // Excel (probes/criteria-operands C1-C4): `<`/`>` match nothing, `<=`/`>=` act as equality.
    // The `#N/A` operand is covered by the same eval cells (C5-C8).
    let t = CalcResult::Boolean(true);
    let text = CalcResult::String(("abc".to_string()).into());
    for op in ["<", ">"] {
        let c = CalcResult::String(format!("{op}TRUE").into());
        let f = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
        assert!(!f(&t) && !f(&text));
    }
    for op in ["<=", ">="] {
        let c = CalcResult::String(format!("{op}TRUE").into());
        let f = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
        assert!(f(&t) && !f(&text) && !f(&CalcResult::Boolean(false)));
    }
}

#[test]
fn test_database_explicit_equals_is_exact_not_prefix() {
    // probes/criteria-operands D18: `ab` matches "abc" by prefix, `=ab` does not.
    let abc = CalcResult::String(("abc".to_string()).into());
    let (plain_c, exact_c) = (
        CalcResult::String(("ab".to_string()).into()),
        CalcResult::String(("=ab".to_string()).into()),
    );
    let plain = build_criteria_with(
        &plain_c,
        en_locale(),
        &year_2026,
        DateSystem::default(),
        CriteriaFlavor::Database,
    );
    let exact = build_criteria_with(
        &exact_c,
        en_locale(),
        &year_2026,
        DateSystem::default(),
        CriteriaFlavor::Database,
    );
    assert!(plain(&abc) && !exact(&abc));
    assert!(exact(&CalcResult::String(("AB".to_string()).into())));
}

#[test]
fn test_build_criteria_is_less_than() {
    let c = CalcResult::String(("<100".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String(("23".to_string()).into())));
    assert!(fn_criteria(&CalcResult::Number(1.0)));
    assert!(!fn_criteria(&CalcResult::Number(101.0)));
}

#[test]
fn test_build_criteria_is_less_wildcard() {
    let c = CalcResult::String(("=D* G*".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::String(
        ("Diarmuid Glynn".to_string()).into()
    )));
    assert!(fn_criteria(&CalcResult::String(
        ("Daniel Gonzalez".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::String(
        ("DanielGonzalez".to_string()).into()
    )));
    assert!(!fn_criteria(&CalcResult::String(
        (" Daniel Gonzalez".to_string()).into()
    )));
}

// Date-string criteria. Excel parses literals like "7/31/2023" into a serial
// (45138 in this case) and applies the comparison numerically against
// date-serial cells. These tests cover all six branches of build_criteria.
//
// The neighbouring serials used below:
//   45137 = 7/30/2023
//   45138 = 7/31/2023  (the criterion target)
//   45139 = 8/1/2023

#[test]
fn test_build_criteria_date_less_than() {
    let c = CalcResult::String(("<7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_less_or_equal() {
    let c = CalcResult::String(("<=7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_greater_than() {
    let c = CalcResult::String((">7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_greater_or_equal() {
    let c = CalcResult::String((">=7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_not_equal() {
    let c = CalcResult::String(("<>7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_equal() {
    let c = CalcResult::String(("7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));

    // Same with a leading "=" prefix.
    let c = CalcResult::String(("=7/31/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}

// Locale coverage for the date-parse fallback.
//
// `parse_date` decides D/M/Y vs M/D/Y by inspecting the locale's
// `date_formats.short`. The tests below pick locales whose short formats are
// known: `en` -> "m/d/yy", `en-GB` -> "dd/mm/yyyy", `es` -> "d/m/yy",
// `de` -> "dd.mm.yy". They guard against three regression classes:
//   1. The locale parameter being dropped or hard-coded to "en".
//   2. The `.` separator path (used by `de`) being broken.
//   3. ISO dates becoming locale-dependent.
//
// Serial numbers used (Excel 1900 system, anchored on 45108 = 2023-07-01):
//   44941 = 2023-01-15
//   44992 = 2023-03-07
//   45110 = 2023-07-03
//   45138 = 2023-07-31

#[test]
fn test_build_criteria_date_dmy_locale_en_gb() {
    // "31/7/2023" is unambiguously a date under en-GB (D/M/Y), but under en
    // (M/D/Y) the would-be month is 31 and the parse must fail.
    let en_gb = get_locale("en-GB").unwrap();
    let c = CalcResult::String(("<31/7/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, en_gb, &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));

    // Under en, the same string is not a parseable date so it falls through to
    // string comparison; numeric cells must never match.
    let fn_criteria_en = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria_en(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria_en(&CalcResult::Number(45138.0)));
}

#[test]
fn test_build_criteria_date_dmy_locale_es() {
    // Spanish is also D/M/Y. Cover equality and >= to exercise different
    // branches than the en-GB test above.
    let es = get_locale("es").unwrap();
    let c = CalcResult::String(("31/7/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, es, &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));

    let c = CalcResult::String((">=31/7/2023".to_string()).into());
    let fn_criteria = build_criteria(&c, es, &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
}

#[test]
fn test_build_criteria_date_dot_separator_locale_de() {
    // German short format is "dd.mm.yy" — the `.` separator is a third path
    // through parse_date, separate from `/` and `-`.
    let de = get_locale("de").unwrap();
    let c = CalcResult::String(("<=31.7.2023".to_string()).into());
    let fn_criteria = build_criteria(&c, de, &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));

    let c = CalcResult::String(("<>31.7.2023".to_string()).into());
    let fn_criteria = build_criteria(&c, de, &year_2026, DateSystem::default());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
}

#[test]
fn test_build_criteria_date_iso_is_locale_independent() {
    // ISO yyyy-mm-dd must parse the same way under every locale. Without this
    // guard, a regression that wrongly applied the D/M/Y swap to ISO dates
    // would slip through.
    let iso = CalcResult::String(("2023-07-31".to_string()).into());
    for id in ["en", "en-GB", "es", "de", "fr", "it"] {
        let loc = get_locale(id).unwrap();
        let fn_criteria = build_criteria(&iso, loc, &year_2026, DateSystem::default());
        assert!(
            fn_criteria(&CalcResult::Number(45138.0)),
            "ISO date should match 45138 under locale {id}"
        );
        assert!(
            !fn_criteria(&CalcResult::Number(45137.0)),
            "ISO date should not match 45137 under locale {id}"
        );
    }
}

#[test]
fn test_build_criteria_date_locale_disambiguation() {
    // "3/7/2023" is ambiguous: en reads March 7 (44992), es reads July 3 (45110).
    // Pin both interpretations to make sure the locale is actually consulted
    // rather than silently defaulted.
    let c = CalcResult::String(("3/7/2023".to_string()).into());

    let fn_en = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(fn_en(&CalcResult::Number(44992.0))); // March 7
    assert!(!fn_en(&CalcResult::Number(45110.0))); // July 3

    let es = get_locale("es").unwrap();
    let fn_es = build_criteria(&c, es, &year_2026, DateSystem::default());
    assert!(!fn_es(&CalcResult::Number(44992.0)));
    assert!(fn_es(&CalcResult::Number(45110.0)));
}

#[test]
fn test_build_criteria_date_localized_month_name() {
    // Spanish abbreviates January as "ene". parse_month consults the locale's
    // months_short list, so this only matches under the matching locale.
    let es = get_locale("es").unwrap();
    let c = CalcResult::String(("=15-ene-2023".to_string()).into());
    let fn_es = build_criteria(&c, es, &year_2026, DateSystem::default());
    assert!(fn_es(&CalcResult::Number(44941.0))); // 2023-01-15
    assert!(!fn_es(&CalcResult::Number(44942.0)));

    // Under en, "ene" is not a known month — parse fails and the criterion
    // falls back to string equality, so no Number cell should match.
    let fn_en = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_en(&CalcResult::Number(44941.0)));
}

// `"<>text"` matches everything that is not that text (numbers, booleans, errors, blanks).
#[test]
fn test_build_criteria_not_equal_text_matches_non_text() {
    let c = CalcResult::String(("<>Banana".to_string()).into());
    let f = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(f(&CalcResult::String(("Apple".to_string()).into())));
    assert!(!f(&CalcResult::String(("BANANA".to_string()).into())));
    assert!(f(&CalcResult::Number(1.0)));
    assert!(f(&CalcResult::Boolean(true)));
    assert!(f(&CalcResult::EmptyCell));
}

// D-functions share the grammar but plain text equality matches by prefix.
#[test]
fn test_build_criteria_database_flavor_prefix_equality() {
    let c = CalcResult::String(("ab".to_string()).into());
    let db = build_criteria_with(
        &c,
        en_locale(),
        &year_2026,
        DateSystem::default(),
        CriteriaFlavor::Database,
    );
    assert!(db(&CalcResult::String(("ABc".to_string()).into())));
    assert!(!db(&CalcResult::String(("xab".to_string()).into())));
    let std = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!std(&CalcResult::String(("abc".to_string()).into())));
    // Operators are unaffected by the flavor.
    let ne = CalcResult::String(("<>ab".to_string()).into());
    let db_ne = build_criteria_with(
        &ne,
        en_locale(),
        &year_2026,
        DateSystem::default(),
        CriteriaFlavor::Database,
    );
    assert!(db_ne(&CalcResult::String(("abc".to_string()).into())));
    // A blank cell only starts with the empty string.
    assert!(!db(&CalcResult::EmptyCell));
}

#[test]
fn test_wildcard_text_matcher_escapes_and_case() {
    let m = wildcard_text_matcher("A~*B?");
    assert!(m(&CalcResult::String(("a*bX".to_string()).into())));
    assert!(!m(&CalcResult::String(("aXbX".to_string()).into())));
    assert!(!m(&CalcResult::Number(1.0)));
    let m = wildcard_text_matcher("a~~b");
    assert!(m(&CalcResult::String(("A~B".to_string()).into())));
    let m = wildcard_text_matcher("a~?");
    assert!(m(&CalcResult::String(("a?".to_string()).into())));
    assert!(!m(&CalcResult::String(("ab".to_string()).into())));
}

// Excel ignores spaces around a numeric operand (probes/criteria-padded-operand);
// spaces before the operator keep the criterion a plain-text one (probes/criteria-gaps C15).
#[test]
fn test_padded_numeric_operand_compares_numerically() {
    let locale = en_locale();
    let hits = |criteria: &str, n: f64| {
        let crit = CalcResult::String(criteria.to_string().into());
        let matcher = build_criteria(&crit, locale, &year_2026, DateSystem::default());
        matcher(&CalcResult::Number(n))
    };
    assert!(hits("> 1", 2.0) && !hits("> 1", 1.0));
    assert!(hits(">1 ", 2.0) && hits("> 1 ", 2.0));
    assert!(hits(">=  1", 1.0) && hits("< 2 ", 1.0) && hits("<= 2", 2.0));
    assert!(hits("= 1", 1.0) && hits("<> 1", 2.0) && !hits("<> 1", 1.0));
    assert!(hits("1 ", 1.0) && hits(" 1", 1.0));
    assert!(!hits(" >1", 2.0));
    assert!(!hits(">\t1", 2.0));
}

// A bare "=" matches a blank cell but not an empty-string cell; "" matches both
// (probes/criteria-grammar C3/C4, probes/criteria-gaps C2/C3).
#[test]
fn test_build_criteria_bare_equals_skips_empty_string() {
    let eq_c = CalcResult::String(("=".to_string()).into());
    let eq = build_criteria(&eq_c, en_locale(), &year_2026, DateSystem::default());
    assert!(!eq(&CalcResult::String(String::new().into())));
    assert!(eq(&CalcResult::EmptyCell));
    let empty_c = CalcResult::String(String::new().into());
    let empty = build_criteria(&empty_c, en_locale(), &year_2026, DateSystem::default());
    assert!(empty(&CalcResult::String(String::new().into())));
    assert!(empty(&CalcResult::EmptyCell));
}

// Excel: criteria equality compares 15-digit values (docs/excel-semantics/numbers.md#criteria-equality).
#[test]
fn test_build_criteria_number_tolerance_is_relative() {
    let one_plus_ulp = 1.0 + f64::EPSILON;
    let fn_criteria = build_criteria(
        &CalcResult::Number(1.0),
        en_locale(),
        &year_2026,
        DateSystem::default(),
    );
    assert!(fn_criteria(&CalcResult::Number(one_plus_ulp)));
    let fn_criteria = build_criteria(
        &CalcResult::Number(0.0),
        en_locale(),
        &year_2026,
        DateSystem::default(),
    );
    assert!(!fn_criteria(&CalcResult::Number(1e-17)));
    assert!(fn_criteria(&CalcResult::Number(0.0)));
    let c = CalcResult::String(("<>1".to_string()).into());
    let fn_criteria = build_criteria(&c, en_locale(), &year_2026, DateSystem::default());
    assert!(!fn_criteria(&CalcResult::Number(one_plus_ulp)));
}

#[test]
fn test_calc_result_ord_uses_15_digit_values() {
    use std::cmp::Ordering;
    let num = CalcResult::Number;
    assert_eq!(num(0.0).cmp(&num(1e-17)), Ordering::Less);
    assert_eq!(num(1.0).cmp(&num(1.0 + f64::EPSILON)), Ordering::Equal);
}

#[test]
fn cmp_excel_precision_total_order_on_nan_and_signed_zero() {
    use crate::number_format::cmp_excel_precision;
    use std::cmp::Ordering;
    assert_eq!(cmp_excel_precision(f64::NAN, f64::NAN), Ordering::Equal);
    assert_eq!(cmp_excel_precision(f64::NAN, 1.0), Ordering::Greater);
    assert_eq!(cmp_excel_precision(1.0, f64::NAN), Ordering::Less);
    assert_eq!(cmp_excel_precision(-f64::NAN, 1.0), Ordering::Greater);
    assert_eq!(cmp_excel_precision(-0.0, 0.0), Ordering::Equal);
}

#[test]
fn text_order_puts_nbsp_right_after_space() {
    // probes/text-order-nbsp-space (excel-graph, T-0244): CHAR(160) sorts after the space and
    // before digits and letters, in `>` and in COUNTIF's `">"` alike.
    let mut model = crate::test::util::new_empty_model();
    model._set("A1", "1");
    model._set("A2", "=\"1 \"");
    model._set("A3", "=\" 1\"");
    model._set("A4", "=CHAR(160)&\"1\"");
    model._set("A5", "=\"1\"&CHAR(160)");
    model._set("A6", "=\"1\"");
    model._set("C1", "=COUNTIF(A1:A6,\">\"&CHAR(160)&\"0\")");
    model._set("C2", "=CHAR(160)&\"1\"<\"1\"");
    model._set("C3", "=\" 1\"<CHAR(160)&\"1\"");
    model._set("C4", "=CHAR(160)&\"1\"<\"0\"");
    model._set("C5", "=\"a b\"<\"a\"&CHAR(160)&\"b\"");
    model._set("C6", "=CHAR(160)&\"1\"=\"1\"");
    model.evaluate();

    // probes/criteria-padded-text C13: every text cell but " 1".
    assert_eq!(model._get_text("C1"), *"4");
    assert_eq!(model._get_text("C2"), *"TRUE");
    assert_eq!(model._get_text("C3"), *"TRUE");
    assert_eq!(model._get_text("C4"), *"TRUE");
    assert_eq!(model._get_text("C5"), *"TRUE");
    assert_eq!(model._get_text("C6"), *"FALSE");
}
