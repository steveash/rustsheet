#![allow(clippy::unwrap_used)]

//! The intersection operator (a space between two references), T-0105. Expected values are
//! Excel's, from gen-m1 (oracle excel-graph, 2026-10-05); see docs/excel-semantics/references.md
//! "Intersection operator (space)", "Disjoint intersection is #NULL!" and "Cross-sheet
//! intersection is #VALUE!".

use crate::model::Model;
use crate::test::util::new_empty_model;

/// Sheet1 (the formulas), `Data` and `My Data`, and `Q'1`, with a few values.
fn model() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model.add_sheet("My Data").unwrap();
    model.add_sheet("Q'1").unwrap();
    model._set("Data!C7", "0.5");
    model._set("Data!C6", "TRUE");
    model._set("Data!C9", "7");
    model._set("My Data!C9", "8");
    model._set("Data!B8", "a");
    model._set("Data!A11", "1");
    model._set("Data!A12", "12");
    model._set("Data!E8", "=1/0");
    model._set("Q'1!E7", "100");
    for (cell, value) in [("Data!B2", "1"), ("Data!C2", "2"), ("Data!B3", "3")] {
        model._set(cell, value);
    }
    model
}

#[test]
fn intersection_gives_the_shared_cell() {
    let mut model = model();
    model._set("A1", "=Data!A1:C7 Data!C7:E12");
    model._set("A2", "=Data!C1:D7 Data!C1:C12 Data!A6:E6");
    model._set("A3", "=Data!C8:B3 Data!B8");
    model._set("A4", "=Data!$A$10:$D$12 Data!A12");
    model._set("A5", "=Data!A7:A7:Data!C11 Data!A11");
    model._set("A6", "='Q''1'!7:7 'Q''1'!E7");
    model._set("A7", "=Data!B:B Data!8:8");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.5");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "a");
    assert_eq!(model._get_text("A4"), "12");
    assert_eq!(model._get_text("A5"), "1");
    assert_eq!(model._get_text("A6"), "100");
    assert_eq!(model._get_text("A7"), "a");
}

#[test]
fn intersection_of_an_empty_cell_is_zero_or_empty_text() {
    let mut model = model();
    model._set("A1", "=Data!Z:Z Data!290:290");
    model._set("A2", "=(Data!D:D Data!2:2)&\"\"");
    model._set("A3", "=Data!D:D Data!2:2&\"x\"");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("A3"), "x");
}

#[test]
fn intersection_binds_tighter_than_arithmetic() {
    let mut model = model();
    model._set("A1", "=(Data!C2:C11 Data!C6:E6)+0.1");
    model._set("A2", "=Data!C2:C11 Data!C9:E9+0.1");
    model._set("A3", "=-Data!C2:C11 Data!C9:E9");
    model._set("A4", "=Data!C2:C11 Data!C9:E9^2");
    model.evaluate();

    // TRUE + 0.1 = 1.1 (gen-m1/4dc8ed1de05a61be Calc!A3).
    assert_eq!(model._get_text("A1"), "1.1");
    assert_eq!(model._get_text("A2"), "7.1");
    assert_eq!(model._get_text("A3"), "-7");
    assert_eq!(model._get_text("A4"), "49");
}

#[test]
fn disjoint_intersection_is_null_error() {
    let mut model = model();
    model._set("A1", "=Data!A3:D4 Data!Z99");
    model._set("A2", "=Data!B4:E11 Data!A1:A3");
    model._set("A3", "=(Data!B4:E11 Data!A1:A3)+1");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NULL!");
    assert_eq!(model._get_text("A2"), "#NULL!");
    assert_eq!(model._get_text("A3"), "#NULL!");
}

#[test]
fn cross_sheet_intersection_is_value_error() {
    let mut model = model();
    // The coordinates overlap (C9), but the areas are on different sheets.
    model._set("A1", "=Data!C5:C10 'My Data'!C9:D9");
    model._set("A2", "=C9 Data!C9");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
}

#[test]
fn intersection_propagates_errors() {
    let mut model = model();
    model._set("A1", "=Data!D2:E8 Data!E8:E12");
    model._set("A2", "=#REF! Data!E8");
    model._set("A3", "=Data!E8 #REF!");
    // A value that is not a reference cannot be intersected: Excel refuses the formula.
    model._set("A4", "=(1/0) Data!E8");
    model._set("A5", "=(1+0) Data!E8");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#DIV/0!");
    assert_eq!(model._get_text("A2"), "#REF!");
    assert_eq!(model._get_text("A3"), "#REF!");
    assert_eq!(model._get_text("A4"), "#ERROR!");
    assert_eq!(model._get_text("A5"), "#ERROR!");
}

#[test]
fn intersection_is_a_reference() {
    let mut model = model();
    model._set("A1", "=SUM(Data!A1:C3 Data!B2:D4)");
    model._set("A2", "=ISREF(Data!A1:C3 Data!B2:D4)");
    model._set("A3", "=ROWS(Data!A1:C3 Data!B2:D4)");
    model._set("A4", "=SUM(B10:C11 C11:D12)");
    model._set("C11", "5");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "5");
}

#[test]
fn multi_cell_intersection_spills_like_a_range() {
    let mut model = model();
    model._set("F1", "=Data!A1:C3 Data!B2:D4");
    model.evaluate();

    assert_eq!(model._get_text("F1"), "1");
    assert_eq!(model._get_text("G1"), "2");
    assert_eq!(model._get_text("F2"), "3");
}

#[test]
fn intersection_formula_text_round_trips() {
    let mut model = model();
    model._set("A1", "=Data!A1:C7 Data!C7:E12");
    model._set("A2", "=(Data!C2:C11 Data!C6:E6)+0.1");
    model._set("A3", "=B1:C3 (C1:C2 C2)");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), "=Data!A1:C7 Data!C7:E12");
    assert_eq!(model._get_formula("A2"), "=(Data!C2:C11 Data!C6:E6)+0.1");
    assert_eq!(model._get_formula("A3"), "=B1:C3 (C1:C2 C2)");
}

#[test]
fn whitespace_between_other_tokens_keeps_its_meaning() {
    let mut model = model();
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A1", "= 1 + 2");
    model._set("A2", "=SUM( B1 , C1 )");
    model._set("A3", "=( B1 )*( C1 )");
    model._set("A4", "=SUM( { 1 , 2 } )");
    model._set("A5", "=B1 -C1");
    model._set("A6", "=B1 & C1");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "6");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(model._get_text("A5"), "-1");
    assert_eq!(model._get_text("A6"), "23");
}

#[test]
fn trailing_input_is_an_error() {
    let mut model = model();
    // Before T-0105 the parser stopped after the first complete expression and dropped the
    // rest: `=1 2` evaluated to 1 and `=SUM(B1))` to SUM(B1).
    model._set("A1", "=1 2");
    model._set("A2", "=SUM(B1))");
    model._set("A3", "=\"a\" \"b\"");
    model._set("A4", "=B1 1");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), "#ERROR!", "{cell}");
    }
}

#[test]
fn ideographic_space_intersection_is_name_error_and_keeps_its_text() {
    // probes/intersection-whitespace H7: `#NAME?` (T-0148).
    let mut model = model();
    model._set("A1", "=A1:A3\u{3000}A2:B2");
    model._set("B9", "=FORMULATEXT(A1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NAME?");
    assert_eq!(model._get_text("B9"), "=A1:A3\u{3000}A2:B2");
}

#[test]
fn ideographic_space_intersection_in_a_group_keeps_its_text() {
    // The first operand of a group, or the group inside a run, is parsed ahead of the
    // intersection (T-3191, T-3197); the name still holds the whole written text.
    let mut model = model();
    for (cell, formula) in [
        ("A1", "=(A1:A3\u{3000}A2:B2)"),
        ("A2", "=((A1:A3)\u{3000}A2:B2)"),
        ("A3", "=(((A1:A3))\u{3000}A2:B2)+1"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula) in [
        ("A1", "=(A1:A3\u{3000}A2:B2)"),
        ("A2", "=((A1:A3)\u{3000}A2:B2)"),
        ("A3", "=(((A1:A3))\u{3000}A2:B2)+1"),
    ] {
        assert_eq!(model._get_text(cell), "#NAME?", "{cell}");
        assert_eq!(model._get_formula(cell), formula, "{cell}");
    }
}

#[test]
fn at_operand_binds_looser_than_the_space() {
    // probes/intersection-operands H10:H11 and probes/intersection-at-operands (T-0149).
    let mut model = model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3"), ("B2", "20")] {
        model._set(cell, value);
    }
    model._set("H10", "=A1 @A1");
    model._set("H11", "=@A1:A3 A2:B2");
    model._set("H12", "=@A1:A3 A2:B2 + 1");
    model._set("H1", "=A1:A3 @A2:B2");
    model.evaluate();
    assert_eq!(model._get_text("H10"), "1");
    assert_eq!(model._get_text("H11"), "2");
    assert_eq!(model._get_text("H12"), "3");
    assert_eq!(model._get_text("H1"), "#VALUE!");
    // Printed as Excel's FORMULATEXT does (probes/intersection-at-formulatext, T-1098).
    assert_eq!(model._get_formula("H10"), "=A1 @A1");
    assert_eq!(model._get_formula("H11"), "=@A1:A3 A2:B2");
}

#[test]
fn at_operand_keeps_a_group_that_is_only_its_leftmost_operand() {
    // probes/at-paren-operands-levels E34:E35 (T-1134).
    let mut model = model();
    model._set("E34", "=@(A1) B1");
    model._set("E35", "=@((A1)) B1");
    model._set("E36", "=@(A1) B1 C1");
    // E37 and E38 are probes/at-group-not-whole-operand E30 and E32 (T-1200); E39 is E31.
    model._set("E37", "=@(A1 B1) C1");
    model._set("E38", "=@(A1):B1 C1");
    model._set("E39", "=@((A1 B1)) C1");
    model.evaluate();
    assert_eq!(model._get_formula("E37"), "=@(A1 B1) C1");
    assert_eq!(model._get_formula("E38"), "=@(A1):B1 C1");
    assert_eq!(model._get_formula("E39"), "=@((A1 B1)) C1");
    assert_eq!(model._get_formula("E34"), "=@(A1) B1");
    assert_eq!(model._get_formula("E35"), "=@((A1)) B1");
    assert_eq!(model._get_formula("E36"), "=@(A1) B1 C1");
}

#[test]
fn at_operand_group_leftmost_shapes_match_the_probe() {
    // probes/at-group-leftmost-shapes E30:E39 (excel-graph, T-1245): Excel prints every form as
    // written. E34 and E35 are asserted separately below.
    let mut model = model();
    let forms = [
        "=@((A1)):B1 C1",
        "=@(A1):B1:C1 D1",
        "=@(A1:B1):C1 D1",
        "=@(A1 B1) C1 D1",
        "=@(A1 B1):C1",
        "=@(A1 B1 C1) D1",
        "=@(A1):OFFSET(B1,0,0) C1",
        // E36: the group is the left corner of a range that is the whole operand (T-1251); the
        // next four are probes/at-group-range-corner-whole E30:E33 (excel-graph).
        "=@(A1):B1",
        "=@(A1:B1):C1",
        "=@((A1)):B1:C1",
        "=@((A1)):B1",
        "=@(A1):B1:C1",
    ];
    for (i, formula) in forms.iter().enumerate() {
        model._set(&format!("E{}", 50 + i), formula);
    }
    model._set("F50", "=@((A1 B1) C1) D1");
    model._set("F51", "=@(A1):(B1) C1");
    model._set("F52", "=@(A1):(B1)");
    model.evaluate();
    for (i, formula) in forms.iter().enumerate() {
        assert_eq!(model._get_formula(&format!("E{}", 50 + i)), *formula);
    }
    // E34 and E35: every group is a `Group` where it was written, a group nested in the leading
    // one (F50) as one right of it (F51, F52), so all print as written (SPEC-0007, T-1343).
    assert_eq!(model._get_formula("F50"), "=@((A1 B1) C1) D1");
    assert_eq!(model._get_formula("F51"), "=@(A1):(B1) C1");
    // probes/at-group-range-corner-whole E34.
    assert_eq!(model._get_formula("F52"), "=@(A1):(B1)");
}

#[test]
fn at_operand_adds_no_level_for_a_nested_inner_group() {
    // T-1167: an inner group that only ends right before the outer `)` adds no level to the `@`
    // operand (only `@((A1))`, a group opening right after the `(`, does). Excel prints every
    // written parenthesis back (probes/at-operand-nested-groups, T-1178), and each is a `Group`
    // where it was written (SPEC-0007, T-1343).
    let mut model = model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("E40", "=@(A1+(B1))");
    model._set("E41", "=@((A1) (B1))");
    model._set("E42", "=@((A1))");
    model._set("E43", "=@(((A1)) B1)");
    model._set("E44", "=@((A1)+(B1))");
    model.evaluate();
    assert_eq!(model._get_formula("E40"), "=@(A1+(B1))");
    assert_eq!(model._get_formula("E41"), "=@((A1) (B1))");
    assert_eq!(model._get_formula("E42"), "=@((A1))");
    assert_eq!(model._get_formula("E43"), "=@(((A1)) B1)");
    assert_eq!(model._get_formula("E44"), "=@((A1)+(B1))");
}

#[test]
fn called_parenthesized_reference_is_ref_error() {
    // probes/paren-call-operands, probes/paren-call-value (Excel refuses `(1)(2)`), T-0149.
    let mut model = model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("H1", "=(A1)(A2)");
    model._set("H2", "=(A1) (A2)");
    model._set("H3", "=(LAMBDA(x,x+1))(2)");
    model._set("H4", "=(1)(2)");
    model._set("H5", "=(IF(TRUE,LAMBDA(x,x*2)))(3)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#REF!");
    assert_eq!(model._get_text("H2"), "#NULL!");
    assert_eq!(model._get_text("H3"), "3");
    assert_eq!(model._get_text("H4"), "#ERROR!");
    assert_eq!(model._get_text("H5"), "6");
    assert_eq!(model._get_formula("H1"), "=(A1)(A2)");
    assert_eq!(model._get_formula("H5"), "=(IF(TRUE,LAMBDA(x,x*2)))(3)");
}

#[test]
fn reference_returning_lookups_are_intersection_operands() {
    // XLOOKUP, TAKE, DROP and TRIMRANGE over a range are references
    // (probes/intersection-function-operands H8, H11, H12, H14; T-0165).
    let mut model = model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "10"),
        ("B2", "20"),
        ("B3", "30"),
    ] {
        model._set(cell, value);
    }
    model._set("H1", "=A1:A3 XLOOKUP(2,A1:A3,A1:B3)");
    model._set("H2", "=A1:A3 TAKE(A2:B3,1)");
    model._set("H3", "=A1:A3 DROP(A1:B2,1)");
    model._set("H4", "=A1:A3 TRIMRANGE(A2:B2)");
    // Counts from the end, and the cells a TRIMRANGE keeps, sit where the range has them.
    model._set("H5", "=A1:A3 TAKE(A1:B3,-1)");
    model._set("H6", "=B1:B3 XLOOKUP(3,A1:A3,A1:B3)");
    model._set("H7", "=A1:A3 DROP(A1:B3,-2,1)");
    model._set("H8", "=A3:A9 TRIMRANGE(A1:B9)");
    // The function's own errors pass through the intersection.
    model._set("H9", "=A1:A3 DROP(A1:B3,5)");
    model._set("H10", "=A1:A3 XLOOKUP(9,A1:A3,A1:B3)");
    model._set("H11", "=A1:A3 TAKE(A1:B3,0)");
    // A count beyond any i64 takes everything and must not overflow.
    model._set("H12", "=A1 TAKE(A1:B3,-1E300)");
    // An array source gives an array, which is not an intersection operand.
    model._set("H13", "=A1:A3 TAKE({1,2},1)");
    model.evaluate();
    for cell in ["H1", "H2", "H3", "H4"] {
        assert_eq!(model._get_text(cell), "2", "{cell}");
    }
    assert_eq!(model._get_text("H5"), "3");
    assert_eq!(model._get_text("H6"), "30");
    assert_eq!(model._get_text("H7"), "#NULL!");
    assert_eq!(model._get_text("H8"), "3");
    assert_eq!(model._get_text("H9"), "#CALC!");
    assert_eq!(model._get_text("H10"), "#N/A");
    assert_eq!(model._get_text("H11"), "#CALC!");
    assert_eq!(model._get_text("H12"), "1");
    assert_eq!(model._get_text("H13"), "#VALUE!");
}

#[test]
fn reference_returning_lookups_keep_their_values_elsewhere() {
    // Outside an intersection the calls still give their values (T-0165).
    let mut model = model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("D1", "=TAKE(A1:B2,-1)");
    model._set("D3", "=SUM(DROP(A1:B2,1))");
    model._set("D4", "=XLOOKUP(2,A1:A2,B1:B2)");
    model._set("D5", "=TAKE(A1:B2,-1E300,1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("E1"), "20");
    assert_eq!(model._get_text("D3"), "22");
    assert_eq!(model._get_text("D4"), "20");
    assert_eq!(model._get_text("D5"), "1");
    assert_eq!(model._get_text("D6"), "2");
}

/// INDEX as an intersection operand is the cell it picks (T-0235).
#[test]
fn index_operand_intersects_the_picked_cell() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C2", "=INDEX(A1:A2,2) A1:A2");
    model._set("C3", "=INDEX(A1:A2,1) A1:A2");
    model._set("C4", "=INDEX(A1:A2,3) A1:A2");
    model.evaluate();
    assert_eq!(model._get_text("C2"), *"2");
    assert_eq!(model._get_text("C3"), *"1");
    assert_eq!(model._get_text("C4"), *"#REF!");
}

/// An aggregate sees the cell HLOOKUP picks as a reference only when the row index is itself a
/// cell reference: COUNT skips a picked FALSE then (gen-m2 60a543744fc4a8f7 A200; probes/
/// lookup-pick-aggregates P1, E1, excel-graph; T-0295); a literal index gives the plain value.
#[test]
fn hlookup_aggregate_argument_is_a_reference_only_for_a_cell_index() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "FALSE");
    model._set("B2", "5");
    model._set("D1", "2");
    model._set("C1", "=COUNT(HLOOKUP(1,A1:B2,D1))");
    model._set("C2", "=COUNT(HLOOKUP(2,A1:B2,D1))");
    model._set("C3", "=COUNT(FALSE)");
    model._set("C4", "=COUNT(HLOOKUP(1,A1:B2,2))");
    model._set("C5", "=COUNT(HLOOKUP(1,A1:B2,(D1)))");
    model._set("C6", "=COUNT(HLOOKUP(1,A1:B2,INT(D1)))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"0");
    assert_eq!(model._get_text("C2"), *"1");
    assert_eq!(model._get_text("C3"), *"1");
    assert_eq!(model._get_text("C4"), *"1");
    assert_eq!(model._get_text("C5"), *"0");
    assert_eq!(model._get_text("C6"), *"1");
}
