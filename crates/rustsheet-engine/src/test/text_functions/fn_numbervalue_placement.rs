#![allow(clippy::unwrap_used)]
//! NUMBERVALUE removes group separators only before the decimal separator, removes spaces, tabs,
//! line feeds and carriage returns anywhere, and reads one leading "%"
//! (docs/excel-semantics/text.md "NUMBERVALUE removes group separators only before the decimal
//! separator"; probes/t3647-numbervalue-separator-placement, probes/t3647-numbervalue-percent-and-controls).

use crate::test::util::new_empty_model;

fn eval(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{formula}");
    }
}

#[test]
fn group_separator_after_the_decimal_separator_stays() {
    eval(&[
        ("A1", r#"=NUMBERVALUE("1.5,2")"#, "#VALUE!"),
        ("A2", r#"=NUMBERVALUE("1.5,")"#, "#VALUE!"),
        ("A3", r#"=NUMBERVALUE(".,5")"#, "#VALUE!"),
        ("A4", r#"=NUMBERVALUE("1,234.5",",",".")"#, "#VALUE!"),
        ("A5", r#"=NUMBERVALUE("1.234,5")"#, "#VALUE!"),
        ("A6", r#"=NUMBERVALUE("1,2.3")"#, "12.3"),
        ("A7", r#"=NUMBERVALUE("1,0E3")"#, "10000"),
        // A digit group separator after the decimal separator reads as that digit.
        ("A8", r#"=NUMBERVALUE("17582","7","8")"#, "1.582"),
        ("A9", r#"=NUMBERVALUE("18275","7","8")"#, "12.5"),
        // A space group separator goes anyway: every space is removed.
        ("A10", r#"=NUMBERVALUE("1.5 2","."," ")"#, "1.52"),
    ]);
}

#[test]
fn tab_line_feed_and_carriage_return_are_removed_other_controls_are_not() {
    eval(&[
        ("A1", r#"=NUMBERVALUE("1"&CHAR(9)&"234.5")"#, "1234.5"),
        ("A2", r#"=NUMBERVALUE("1"&CHAR(10)&"234.5")"#, "1234.5"),
        ("A3", r#"=NUMBERVALUE("1"&CHAR(13)&"234.5")"#, "1234.5"),
        ("A4", r#"=NUMBERVALUE("1"&CHAR(11)&"234")"#, "#VALUE!"),
        ("A5", r#"=NUMBERVALUE("1"&CHAR(160)&"234.5")"#, "#VALUE!"),
        (
            "A6",
            r#"=NUMBERVALUE("1"&UNICHAR(12288)&"234.5")"#,
            "#VALUE!",
        ),
        ("A7", r#"=NUMBERVALUE("12:30"&CHAR(10)&"PM")"#, "#VALUE!"),
        ("A8", r#"=NUMBERVALUE(CHAR(9)&CHAR(13))"#, "0"),
    ]);
}

#[test]
fn one_leading_percent_sign_counts() {
    eval(&[
        ("A1", r#"=NUMBERVALUE("%1.5")"#, "0.015"),
        ("A2", r#"=NUMBERVALUE("%1.5%")"#, "0.00015"),
        ("A3", r#"=NUMBERVALUE("-%1.5")"#, "-0.015"),
        ("A4", r#"=NUMBERVALUE("%-1.5")"#, "-0.015"),
        ("A5", r#"=NUMBERVALUE("(%1.5)")"#, "-0.015"),
        ("A6", r#"=NUMBERVALUE("%1,234.5")"#, "12.345"),
        ("A7", r#"=NUMBERVALUE("%%1.5")"#, "#VALUE!"),
        ("A8", r#"=NUMBERVALUE("%$1.5")"#, "#VALUE!"),
        ("A9", r#"=NUMBERVALUE("$%1.5")"#, "#VALUE!"),
        ("A10", r#"=NUMBERVALUE("%")"#, "0"),
        ("A11", r#"=NUMBERVALUE("%%")"#, "0"),
    ]);
}
