#![allow(clippy::unwrap_used)]

//! IMSUM and IMPRODUCT take their arguments whole, and the IM* functions refuse a boolean
//! (T-1384; probes/eng-argument-modes A60:A83, A89, probes/eng-imsum-improduct-whole;
//! docs/excel-semantics/engineering.md "IMSUM and IMPRODUCT take their arguments whole").

use crate::test::util::new_empty_model;

/// The probe's data: M1 "1+2i", M2 3, M3 blank, M4 TRUE, M5 "x", M6 "2-i", M7 =1/0, M8 "3+4j",
/// M9 ="".
fn set_data(model: &mut crate::Model) {
    model._set("M1", "1+2i");
    model._set("M2", "3");
    model._set("M4", "TRUE");
    model._set("M5", "x");
    model._set("M6", "2-i");
    model._set("M7", "=1/0");
    model._set("M8", "3+4j");
    model._set("M9", "=\"\"");
}

fn check(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    set_data(&mut model);
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn ranges_and_arrays_are_folded_whole() {
    check(&[
        // probes/eng-argument-modes A60:A83.
        ("=IMSUM(M1:M2)", "4+2i"),
        ("=IMSUM(M1:M3)", "4+2i"),
        ("=IMSUM({\"1+2i\",\"3\"})", "4+2i"),
        ("=ROWS(IMSUM({\"1+2i\";\"3\"}))", "1"),
        ("=IMSUM(\"1+2i\",3)", "4+2i"),
        ("=IMSUM(\"1+2i\",M2,\"2-i\")", "6+i"),
        ("=IMSUM(M1:M2,M6)", "6+i"),
        ("=IMPRODUCT(M1:M2)", "3+6i"),
        ("=IMPRODUCT({\"1+2i\",\"3\"})", "3+6i"),
        ("=ROWS(IMPRODUCT({\"1+2i\";\"3\"}))", "1"),
        ("=IMPRODUCT(\"1+2i\",M2,\"2-i\")", "12+9i"),
        ("=IMPRODUCT(M1:M2,M6)", "12+9i"),
        // E42, E50, B42, C50: a range or array next to a scalar does not lift.
        ("=ROWS(IMPRODUCT(M1:M2,\"1-2i\"))", "1"),
        ("=ROWS(IMSUM({\"3+4i\";\"3+4i\"},\"1-2i\"))", "1"),
        ("=ROWS(IMPRODUCT(\"3+4i\",{\"1-2i\";\"1-2i\"}))", "1"),
        // probes/eng-imsum-improduct-whole B20-B24.
        ("=IMSUM(IF(TRUE,M1:M2))", "4+2i"),
        ("=IMSUM(M1)", "1+2i"),
        ("=IMPRODUCT(M2)", "3"),
        ("=IMSUM({\"1+2i\";\"3\"},{\"1\";\"2\"})", "7+2i"),
        ("=IMSUM({1,2;3,4})", "10"),
    ]);
}

#[test]
fn a_blank_cell_is_zero_and_an_empty_argument_is_skipped() {
    check(&[
        ("=IMPRODUCT(M1:M3)", "0"),
        // Q1:Q3 lies past the used area (eng-imsum-improduct-whole B11, B12).
        ("=IMPRODUCT(M1:M2,Q1:Q3)", "0"),
        ("=IMSUM(M1:M2,Q1:Q3)", "4+2i"),
        // B17, B18.
        ("=IMSUM(M1,)", "1+2i"),
        ("=IMPRODUCT(M1,)", "1+2i"),
        // B28: the blank does not stop the walk.
        ("=IMPRODUCT(M1:M3,M5)", "#NUM!"),
    ]);
}

#[test]
fn the_first_bad_item_in_order_is_the_result() {
    check(&[
        // probes/eng-argument-modes A62:A65, A69, A74:A77, A81.
        ("=IMSUM(M1:M2,M4)", "#VALUE!"),
        ("=IMSUM(M1:M2,M5)", "#NUM!"),
        ("=IMSUM(M1:M6)", "#VALUE!"),
        ("=IMSUM(M5:M6)", "#NUM!"),
        ("=IMSUM(\"1+2i\",TRUE)", "#VALUE!"),
        ("=IMPRODUCT(M1:M2,M4)", "#VALUE!"),
        ("=IMPRODUCT(M1:M2,M5)", "#NUM!"),
        ("=IMPRODUCT(M1:M6)", "#VALUE!"),
        ("=IMPRODUCT(M5:M6)", "#NUM!"),
        ("=IMPRODUCT(\"1+2i\",TRUE)", "#VALUE!"),
        // probes/eng-imsum-improduct-whole B1:B10, B26, B27.
        ("=IMSUM({\"1+2i\",TRUE})", "#VALUE!"),
        ("=IMPRODUCT({\"1+2i\",TRUE})", "#VALUE!"),
        ("=IMSUM(M1:M2,M7)", "#DIV/0!"),
        ("=IMSUM(M5,M7)", "#NUM!"),
        ("=IMSUM(M7,M5)", "#DIV/0!"),
        ("=IMSUM(M4,M5)", "#VALUE!"),
        ("=IMSUM(M5,M4)", "#NUM!"),
        ("=IMSUM({\"x\",TRUE})", "#NUM!"),
        ("=IMSUM(\"x\",NA())", "#NUM!"),
        ("=IMSUM(M1,M8)", "#VALUE!"),
        ("=IMSUM(M5:M7)", "#NUM!"),
        ("=IMSUM(M7,M4)", "#DIV/0!"),
        // B13:B16, B25: empty text is parsed, so `#NUM!`.
        ("=IMPRODUCT(M1,\"\")", "#NUM!"),
        ("=IMSUM(M1,\"\")", "#NUM!"),
        ("=IMSUM(M1,M9)", "#NUM!"),
        ("=IMPRODUCT(M1,M9)", "#NUM!"),
        ("=IMPRODUCT({\"1+2i\",\"\"})", "#NUM!"),
        // B19: a union is refused.
        ("=IMSUM((M1,M6))", "#VALUE!"),
    ]);
}

#[test]
fn the_im_functions_refuse_a_boolean() {
    check(&[
        // probes/eng-argument-modes A89, probes/eng-imsum-improduct-whole C1:C15.
        ("=IMABS(TRUE)", "#VALUE!"),
        ("=IMREAL(TRUE)", "#VALUE!"),
        ("=IMSUB(TRUE,\"1\")", "#VALUE!"),
        ("=IMSUB(\"1\",TRUE)", "#VALUE!"),
        ("=IMDIV(\"1\",TRUE)", "#VALUE!"),
        ("=IMPOWER(TRUE,2)", "#VALUE!"),
        ("=IMPOWER(\"2i\",TRUE)", "#VALUE!"),
        ("=IMCOS(TRUE)", "#VALUE!"),
        ("=IMABS(M4)", "#VALUE!"),
        ("=INDEX(IMREAL({TRUE;\"3\"}),1)", "#VALUE!"),
        ("=INDEX(IMREAL({TRUE;\"3\"}),2)", "3"),
        ("=IMCONJUGATE(FALSE)", "#VALUE!"),
        ("=COMPLEX(TRUE,1)", "#VALUE!"),
        ("=IMSQRT(M4)", "#VALUE!"),
        ("=IMPOWER(\"2i\",M4)", "#VALUE!"),
        ("=IMSUB(M4,\"1\")", "#VALUE!"),
        // Controls: a number is read as its digits.
        ("=IMABS(3)", "3"),
        ("=COMPLEX(3,4)", "3+4i"),
    ]);
}

/// The values read across ranges and arrays are capped (directive 5): a call over more cells
/// than `MAX_LIFTED_CELL_VISITS` is `#ERROR!` before it walks them. A whole column is clipped to
/// the used area, so it reads only the data.
#[test]
fn the_cells_read_are_capped() {
    let mut model = new_empty_model();
    model._set("A1", "1+2i");
    model._set("A2", "3");
    // 2^20 rows x 256 columns = 2^28 cells, above the 2^27 cap; nothing is filled there.
    model._set("C1", "=IMSUM(XA1:XFD1048576)");
    // Two whole columns, clipped to the two used rows.
    model._set("C2", "=IMSUM(A:A,A:A)");
    model._set("C3", "=IMPRODUCT(A:A)");
    // The used area reaches row 1,048,576: 1,048,576 x 130 columns = 2^27 + 2^21 cells.
    model._set("E1048576", "1");
    model._set("C4", "=IMSUM(A1:DZ1048576)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"8+4i");
    // A:A holds blank cells past A2, so the product is 0.
    assert_eq!(model._get_text("C3"), *"0");
    assert_eq!(model._get_text("C4"), *"#ERROR!");
}
