#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_concatenate_args_number() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCATENATE()");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn fn_concatenate() {
    let mut model = new_empty_model();
    model._set("A1", "Hello");
    model._set("A2", " my ");
    model._set("A3", "World");

    model._set("B1", r#"=CONCATENATE(A1, A2, A3, "!")"#);
    model._set("C2", r#"=CONCATENATE(@A1:A3, "!")"#);
    // An array argument lifts element-wise and spills: ["Hello!", " my!", "World!"].
    model._set("B2", r#"=CONCATENATE(A1:A3, "!")"#);
    model._set("D3", r#"=CONCAT(A1:A3, "!")"#);

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Hello my World!");
    assert_eq!(model._get_text("B2"), *"Hello!");
    assert_eq!(model._get_text("B3"), *" my !");
    assert_eq!(model._get_text("B4"), *"World!");
    assert_eq!(model._get_text("D3"), *"Hello my World!");
    assert_eq!(model._get_text("C2"), *" my !");
}

#[test]
fn fn_concatenate_lifts_over_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("A2", "b");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("D1", "=CONCATENATE(A1:A2,\"x\")");
    model._set("F1", "=CONCATENATE(A1:A2,B1:B2)");
    model._set("H1", "=CONCATENATE(\"p\",\"q\",1)");
    model._set("J1", "=CONCAT(A1:A2)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"ax");
    assert_eq!(model._get_text("D2"), *"bx");
    assert_eq!(model._get_text("F1"), *"a1");
    assert_eq!(model._get_text("F2"), *"b2");
    assert_eq!(model._get_text("H1"), *"pq1");
    assert_eq!(model._get_text("J1"), *"ab");
}

#[test]
fn fn_concatenate_and_ampersand_truncate_at_32767_chars() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("B1", "=REPT(\"a\",20000)");
    model._set("C1", "=LEN(CONCATENATE(A1:A2,B1,B1))");
    model._set("E1", "=LEN(CONCATENATE(B1,\"b\"))");
    model._set("D1", "=LEN(REPT(\"a\",32767)&\"b\")");
    model._set("D2", "=LEN(CONCATENATE(REPT(\"a\",32767),\"b\"))");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"32767");
    assert_eq!(model._get_text("E1"), *"20001");
    assert_eq!(model._get_text("D1"), *"32767");
    assert_eq!(model._get_text("D2"), *"32767");
}

#[test]
fn fn_concatenate_limit_counts_utf16_units() {
    // probes/concatenate-limit-spill E7–E9 (excel-graph): the 32,767 limit is in UTF-16 code
    // units, so an emoji (a surrogate pair) counts two.
    let mut model = new_empty_model();
    model._set("A1", "=CONCATENATE(REPT(\"😀\",16383),\"ab\")");
    model._set("A2", "=REPT(\"😀\",16383)&\"😀\"");
    model._set("A3", "=CONCATENATE(REPT(\"é\",20000),REPT(\"é\",20000))");

    model.evaluate();

    // 32,766 units of emoji + "a"; "b" falls past the limit.
    let a1 = model._get_text("A1");
    assert_eq!(a1.encode_utf16().count(), 32767);
    assert!(a1.ends_with("😀a"));
    // The last emoji straddles the limit; Excel keeps its high half (E11), which the lossy
    // text view shows as one U+FFFD unit.
    assert_eq!(model._get_text("A2").encode_utf16().count(), 32767);
    // "é" is one UTF-16 unit (two UTF-8 bytes).
    assert_eq!(model._get_text("A3").chars().count(), 32767);
}

// Oracle: probes/text-result-limit C8, C9 (excel-graph): CONCAT and TEXTJOIN text over 32767
// chars is #CALC!. The exactly-32767 cases are hand-written, pending an oracle probe (T-0262).
#[test]
fn concat_and_textjoin_over_32767_chars_are_calc_error() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN(CONCAT(REPT(\"a\",32767),\"b\"))");
    model._set("A2", "=LEN(CONCAT(REPT(\"a\",32766),\"b\"))");
    model._set("A3", "=LEN(TEXTJOIN(\",\",TRUE,REPT(\"a\",32767),\"b\"))");
    model._set("A4", "=LEN(TEXTJOIN(\",\",TRUE,REPT(\"a\",32765),\"b\"))");
    model._set("A5", "=LEN(TEXTJOIN(\",\",TRUE,REPT(\"a\",32766),\"b\"))");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#CALC!");
    assert_eq!(model._get_text("A2"), *"32767");
    assert_eq!(model._get_text("A3"), *"#CALC!");
    assert_eq!(model._get_text("A4"), *"32767");
    assert_eq!(model._get_text("A5"), *"#CALC!");
}

// The limit counts UTF-16 units, not bytes: 16383 + 16384 two-byte chars are 65534 bytes but
// exactly 32767 units, so they fit; one more char does not.
#[test]
fn concat_and_textjoin_limit_with_multibyte_and_range_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=REPT(\"é\",16383)");
    model._set("A2", "=REPT(\"é\",16384)");
    model._set("B1", "=LEN(CONCAT(A1,\"é\"))");
    model._set("B2", "=LEN(CONCAT(A1:A2))");
    model._set("B3", "=LEN(TEXTJOIN(\"\",TRUE,A1,A1))");
    model._set("B4", "=LEN(TEXTJOIN(\"\",TRUE,A1:A2))");
    model._set("B5", "=LEN(CONCAT(A1:A2,\"é\"))");
    model._set("B6", "=LEN(TEXTJOIN(\"\",TRUE,A1:A2,\"é\"))");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"16384");
    assert_eq!(model._get_text("B2"), *"32767");
    assert_eq!(model._get_text("B3"), *"32766");
    assert_eq!(model._get_text("B4"), *"32767");
    assert_eq!(model._get_text("B5"), *"#CALC!");
    assert_eq!(model._get_text("B6"), *"#CALC!");
}

// Oracle: probes/text-result-limit C18-C20 (excel-graph): an error argument after the limit wins
// over `#CALC!`, but an error in a later cell of a range does not: the walk has already gone past
// the limit, so the length error comes first.
#[test]
fn concat_and_textjoin_error_after_the_limit() {
    let mut model = new_empty_model();
    model._set("B1", "b");
    model._set("B2", "=NA()");
    model._set("A1", "=CONCAT(REPT(\"a\",32767),\"b\",NA())");
    model._set("A2", "=TEXTJOIN(\",\",TRUE,REPT(\"a\",32767),\"b\",NA())");
    model._set("A3", "=TEXTJOIN(\",\",TRUE,REPT(\"a\",32767),B1:B2)");
    // Under the limit the cell error is the result, as ever.
    model._set("A4", "=TEXTJOIN(\",\",TRUE,REPT(\"a\",100),B1:B2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#CALC!");
    assert_eq!(model._get_text("A4"), *"#N/A");
}

#[test]
fn concat_joins_array_arguments_row_by_row() {
    // gen-m2/d0c29dd5c530e28c Calc!A246: CONCAT(MID(array,7,0)) is "".
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT({\"a\",\"b\";\"c\",\"d\"})");
    model._set("A2", "=CONCAT(MID({\"abc\",\"defghijk\"},7,0))");
    model._set("A3", "=CONCAT({\"a\",1},\"z\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"abcd");
    assert_eq!(model._get_text("A2"), *"");
    assert_eq!(model._get_text("A3"), *"a1z");
}

// Excel keeps the high half of a surrogate pair the 32,767-unit limit cuts
// (probes/concatenate-limit-spill E8, E9, E11); arguments after the cut change nothing, but an
// error argument after it still wins.
#[test]
fn concatenation_keeps_a_lone_high_surrogate_at_the_limit() {
    let mut model = new_empty_model();
    model._set("A1", r#"=LEN(CONCATENATE(REPT("😀",16383),"😀"))"#);
    model._set("A2", r#"=LEN(CONCATENATE(REPT("a",32766),"😀"))"#);
    model._set("A3", r#"=LEN(REPT("😀",16383)&"😀")"#);
    model._set("A4", r#"=LEN(CONCATENATE(REPT("a",32766),"😀","zz",1))"#);
    model._set("A5", r#"=CONCATENATE(REPT("a",32766),"😀",NA())"#);
    model._set("A6", r#"=LEN("😀"&REPT("a",32766))"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"32767");
    assert_eq!(model._get_text("A2"), *"32767");
    assert_eq!(model._get_text("A3"), *"32767");
    assert_eq!(model._get_text("A4"), *"32767");
    assert_eq!(model._get_text("A5"), *"#N/A");
    assert_eq!(model._get_text("A6"), *"32767");
}
