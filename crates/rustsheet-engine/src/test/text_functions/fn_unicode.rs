#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(\"1,00\")");
    model._set("A2", "=UNICODE(\"1\")");
    model._set("A3", "=UNICODE(1)");
    model._set("A4", "=UNICODE(\"T\")");
    model._set("A5", "=UNICODE(\"TRUE\")");
    model._set("A6", "=UNICODE(TRUE)");
    model._set("A7", "=UNICODE(FALSE)");
    model._set("A8", "=UNICODE(\"の\")");
    model._set("A9", "=UNICODE(\" \")");

    model._set("A10", "=_xlfn.UNICODE(\"T\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"49");
    assert_eq!(model._get_text("A2"), *"49");
    assert_eq!(model._get_text("A3"), *"49");
    assert_eq!(model._get_text("A4"), *"84");
    assert_eq!(model._get_text("A5"), *"84");
    assert_eq!(model._get_text("A6"), *"84");
    assert_eq!(model._get_text("A7"), *"70");
    assert_eq!(model._get_text("A8"), *"12398");
    assert_eq!(model._get_text("A9"), *"32");
    assert_eq!(model._get_text("A10"), *"84");
}

#[test]
fn test_error_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(\"\")");
    model._set("A2", "=UNICODE(#CALC!)");
    model._set("A3", "=UNICODE(#NAME?)");
    model._set("A4", "=UNICODE(#VALUE!)");
    model._set("A5", "=UNICODE(#REF!)");
    model._set("A6", "=UNICODE(#DIV/0!)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#CALC!");
    assert_eq!(model._get_text("A3"), *"#NAME?");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#REF!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE()");
    model._set("A2", "=UNICODE(\"B\",\"A\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

/// UNICODE lifts over an array or a range, as CODE does, so a lifted INDIRECT spills and a
/// blocked spill is `#SPILL!` (gen-m9/675763979c1defb6 Calc!A25; T-2090). The element values
/// are each element's scalar UNICODE (not oracle-probed).
#[test]
fn unicode_lifts_over_arrays_and_ranges() {
    let mut model = new_empty_model();
    model._set("A1", "A");
    model._set("A2", "b");
    model._set("C1", "=ROWS(UNICODE(INDIRECT({\"A1\";\"A2\"})))");
    model._set("D1", "=UNICODE(A1:A3)");
    model._set("E1", "=UNICODE({\"A\",\"\"})");
    model._set("G1", "=UNICODE(A1:A2)");
    model._set("G2", "blocker");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"2");
    assert_eq!(model._get_text("D1"), *"65");
    assert_eq!(model._get_text("D2"), *"98");
    assert_eq!(model._get_text("D3"), *"#VALUE!");
    assert_eq!(model._get_text("E1"), *"65");
    assert_eq!(model._get_text("F1"), *"#VALUE!");
    assert_eq!(model._get_text("G1"), *"#SPILL!");
}
