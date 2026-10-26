#![allow(clippy::panic)]

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::types::CellReferenceRC;

#[test]
fn exp_order() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse("(1 + 2)^3  + 4", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "(1 + 2)^3  + 4"
    );

    let t = parser.parse("(C5 + 3)^R4", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "(C5 + 3)^R4"
    );

    let t = parser.parse("(C5 + 3)^(R4*6)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "(C5 + 3)^(R4*6)"
    );

    // Written groups print back (SPEC-0007).
    let t = parser.parse("(C5)^(R4)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "(C5)^(R4)"
    );

    let t = parser.parse("(5)^(4)", &cell_reference);
    assert_eq!(to_english_localized_string(&t, &cell_reference), "(5)^(4)");
}

#[test]
fn correct_parenthesis() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());

    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };

    let t = parser.parse("-(1 + 1)", &cell_reference);
    assert_eq!(to_english_localized_string(&t, &cell_reference), "-(1 + 1)");

    let t = parser.parse("1 - (3 + 4)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "1 - (3 + 4)"
    );

    let t = parser.parse("-(1.05*(0.0284 + 0.0046) - 0.0284)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "-(1.05*(0.0284 + 0.0046) - 0.0284)"
    );

    let t = parser.parse("1 + (3+5)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "1 + (3+5)"
    );

    let t = parser.parse("1 - (3+5)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "1 - (3+5)"
    );

    let t = parser.parse("(1 - 3) - (3+5)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "(1 - 3) - (3+5)"
    );

    let t = parser.parse("1 + (3<5)", &cell_reference);
    assert_eq!(
        to_english_localized_string(&t, &cell_reference),
        "1 + (3<5)"
    );
}

#[test]
fn names_with_backslash_round_trip() {
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    for formula in [
        "IF(HSI\\10_1___val<>\"\",HSI\\10_1___val,\"\")",
        "A\\B+1",
        "\\name",
    ] {
        let t = parser.parse(formula, &cell_reference);
        assert_eq!(to_english_localized_string(&t, &cell_reference), formula);
    }
}

#[test]
fn comparison_operands_keep_their_parentheses() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    // T-0151: the printed text must parse back to the same tree.
    for formula in [
        "\"a\"&(2<3)",
        "(2<3)&\"b\"",
        "1&(2<3)&1",
        "\"a\"&(2=3)&\"b\"",
        "1<(2<3)",
        "1<2<3",
    ] {
        let t = parser.parse(formula, &cell_reference);
        let printed = to_english_localized_string(&t, &cell_reference);
        assert_eq!(printed, formula);
        let again = parser.parse(&printed, &cell_reference);
        assert_eq!(again, t, "{formula}");
    }
}

#[test]
fn concatenation_operands_of_arithmetic_keep_their_parentheses() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    // T-0157: `&` binds looser than `+ - * /`; the printed text must parse back to the same tree.
    for formula in [
        "(\"a\"&1)+1",
        "1+(\"a\"&1)",
        "(\"a\"&1)*2",
        "2/(\"a\"&1)",
        "1-(\"a\"&1)",
        "(\"a\"&1)-1",
    ] {
        let t = parser.parse(formula, &cell_reference);
        let printed = to_english_localized_string(&t, &cell_reference);
        assert_eq!(printed, formula);
        let again = parser.parse(&printed, &cell_reference);
        assert_eq!(again, t, "{formula}");
    }
}
