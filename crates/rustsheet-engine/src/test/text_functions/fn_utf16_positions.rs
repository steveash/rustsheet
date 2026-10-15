#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel counts LEN, MID, FIND, SEARCH and REPLACE positions in UTF-16 code units, LEFT and RIGHT
// in characters (probes/text-surrogate-positions, excel-graph; docs/excel-semantics/text.md
// "Text positions count UTF-16 code units, except in LEFT and RIGHT").

#[test]
fn positions_count_utf16_units() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN(\"a😀b\")");
    model._set("A2", "=MID(\"a😀b\",2,2)");
    model._set("A3", "=MID(\"a😀b\",4,1)");
    model._set("A4", "=FIND(\"b\",\"😀b😀b\",4)");
    model._set("A5", "=FIND(\"b\",\"😀b\",2)");
    model._set("A6", "=SEARCH(\"?b\",\"😀b\")");
    model._set("A7", "=SEARCH(\"a?b\",\"a😀b\")");
    model._set("A8", "=SEARCH(\"a??b\",\"a😀b\")");
    model._set("A9", "=REPLACE(\"a😀b\",2,2,\"x\")");
    model._set("A10", "=REPLACE(\"a😀b\",4,1,\"x\")");
    model._set("A11", "=FIND(\"😀\",\"a😀\",3)");
    model._set("A12", "=LENB(\"𝐀\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "😀");
    assert_eq!(model._get_text("A3"), "b");
    assert_eq!(model._get_text("A4"), "6");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "#VALUE!");
    assert_eq!(model._get_text("A8"), "1");
    assert_eq!(model._get_text("A9"), "axb");
    assert_eq!(model._get_text("A10"), "a😀x");
    assert_eq!(model._get_text("A11"), "#VALUE!");
    assert_eq!(model._get_text("A12"), "2");
}

#[test]
fn left_and_right_count_code_points() {
    let mut model = new_empty_model();
    model._set("A1", "=LEFT(\"a😀b\",3)");
    model._set("A2", "=RIGHT(\"a😀b\",2)");
    model._set("A3", "=LEFT(\"👍🏽x\",1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a😀b");
    assert_eq!(model._get_text("A2"), "😀b");
    assert_eq!(model._get_text("A3"), "👍");
}

#[test]
fn split_pair_keeps_the_lone_half() {
    // Excel keeps the lone surrogate a cut leaves (probes/text-surrogate-positions A11, A12, A29);
    // `_get_text` shows it through the lossy view as U+FFFD, LEN counts it as one unit.
    let mut model = new_empty_model();
    model._set("A1", "=MID(\"a😀b\",3,2)");
    model._set("A2", "=MID(\"a😀b\",2,1)");
    model._set("A3", "=REPLACE(\"a😀b\",3,1,\"x\")");
    model._set("B1", "=LEN(A1)");
    model._set("B2", "=LEN(A2)");
    model._set("B3", "=LEN(A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "\u{FFFD}b");
    assert_eq!(model._get_text("A2"), "\u{FFFD}");
    assert_eq!(model._get_text("A3"), "a\u{FFFD}xb");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "1");
    assert_eq!(model._get_text("B3"), "4");
}

#[test]
fn find_start_one_past_the_end() {
    let mut model = new_empty_model();
    model._set("A1", "=FIND(\"\",\"abc\",4)");
    model._set("A2", "=FIND(\"\",\"abc\",5)");
    model._set("A3", "=FIND(\"\",\"\",1)");
    model._set("A4", "=FIND(\"c\",\"abc\",4)");
    model._set("A5", "=SEARCH(\"\",\"abc\",4)");
    model._set("A6", "=FIND(\"\",\"😀b\",4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "4");
}

#[test]
fn left_right_unicode_read_lone_surrogates_exactly() {
    // probes/text-surrogate-positions A38, B6, B7; probes/text-lone-surrogates A3, A4: LEFT and
    // RIGHT keep a lone surrogate as one character, UNICODE of a lone high one is #VALUE!.
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(LEFT(MID(\"a😀b\",3,2),1))");
    model._set("A2", "=UNICODE(RIGHT(MID(\"a😀b\",3,2),2))");
    model._set("A3", "=LEN(LEFT(MID(\"a😀b\",3,2),1))");
    model._set("A4", "=UNICODE(MID(\"😀\",1,1))");
    model._set("A5", "=UNICODE(MID(\"😀\",2,1))");
    model._set("A6", "=LEN(RIGHT(MID(\"a😀b\",2,1),1))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "56832");
    assert_eq!(model._get_text("A2"), "56832");
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "56832");
    assert_eq!(model._get_text("A6"), "1");
}

#[test]
fn lone_surrogates_compare_by_code_unit() {
    // probes/text-lone-surrogates A10-A28 (excel-graph): a lone half is never U+FFFD, H<L, and
    // H<U+FFFD, L<U+FFFD are FALSE.
    let h = "MID(\"😀\",1,1)";
    let l = "MID(\"😀\",2,1)";
    let cases = [
        (format!("=EXACT({h},UNICHAR(65533))"), "FALSE"),
        (format!("=EXACT({h},{l})"), "FALSE"),
        (format!("=EXACT({h},{h})"), "TRUE"),
        (format!("={h}=UNICHAR(65533)"), "FALSE"),
        (format!("={h}={l}"), "FALSE"),
        (format!("={h}<{l}"), "TRUE"),
        (format!("={l}<{h}"), "FALSE"),
        (format!("={h}<\"a\""), "FALSE"),
        (format!("={h}<\"😀\""), "TRUE"),
        (format!("=\"😀\"<{l}"), "TRUE"),
        (format!("={h}<UNICHAR(65533)"), "FALSE"),
        (format!("={l}<UNICHAR(65533)"), "FALSE"),
        (
            format!("=MATCH({h},{{\"x\";\"😀\";\"\u{fffd}\"}},0)"),
            "#N/A",
        ),
        (format!("=MATCH({l},{{\"x\";\"\u{fffd}\"}},0)"), "#N/A"),
        (format!("=EXACT(UPPER({h}),{h})"), "TRUE"),
        (format!("=LEN(TRIM(\" \"&{h}&\"  \"&{l}&\" \"))"), "3"),
        (format!("=LEN(CLEAN({l}&CHAR(7)&{h}))"), "2"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model._set("A1", &formula);
        model.evaluate();
        assert_eq!(model._get_text("A1"), expected, "{formula}");
    }
}

#[test]
fn match_of_a_lone_half_pattern_ignores_case() {
    // A lone-half pattern compares like `=`, case-insensitively (T-1585 review).
    let mut model = new_empty_model();
    model._set("B1", "x");
    model._set("B2", "=\"a\"&MID(\"😀\",1,1)");
    model._set("A1", "=MATCH(\"A\"&MID(\"😀\",1,1),B1:B2,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn find_search_substitute_split_pairs_with_a_lone_half() {
    // probes/text-lone-surrogates A43-A57 (excel-graph): a lone half as find_text or old_text
    // matches inside a pair, and never matches U+FFFD.
    let h = "MID(\"😀\",1,1)";
    let l = "MID(\"😀\",2,1)";
    let cases = [
        (format!("=FIND({h},\"a😀b\")"), "2"),
        (format!("=FIND({l},\"a😀b\")"), "3"),
        (format!("=SEARCH({h},\"a😀b\")"), "2"),
        (format!("=SEARCH({l},\"a😀b\")"), "3"),
        (format!("=FIND({l},{h}&{l})"), "2"),
        (format!("=FIND(\"😀\",{h}&{l})"), "1"),
        (format!("=FIND(UNICHAR(65533),{h})"), "#VALUE!"),
        (format!("=SEARCH(UNICHAR(65533),{l})"), "#VALUE!"),
        (format!("=ISNUMBER(SEARCH({h}&\"*\",\"a😀b\"))"), "TRUE"),
        (format!("=UNICODE(SUBSTITUTE(\"😀\",{h},\"x\"))"), "120"),
        (format!("=LEN(SUBSTITUTE(\"😀\",{l},\"x\"))"), "2"),
        (format!("=LEN(SUBSTITUTE(\"a😀b😀\",{h},\"x\"))"), "6"),
        (format!("=LEN(SUBSTITUTE(\"😀\",{h}&{l},\"x\"))"), "1"),
        (format!("=LEN(SUBSTITUTE({h},{h},\"x\"))"), "1"),
        // The result limit applies to the lone-surrogate path too (text.md "SUBSTITUTE result longer").
        (format!("=SUBSTITUTE(REPT(\"a\",32766)&{h},{h},\"xx\")"), "#VALUE!"),
        (format!("=LEN(SUBSTITUTE(REPT(\"a\",32766)&{h},{h},\"x\"))"), "32767"),
        // A long near-miss pattern stays linear.
        (format!("=SUBSTITUTE(REPT(\"a\",16000)&{l},REPT(\"a\",8000)&\"b\",\"x\")=REPT(\"a\",16000)&{l}"), "TRUE"),
        (format!("=LEN(SUBSTITUTE(\"a\"&{h}&\"b\"&{h},{h},\"-\",2))"), "4"),
        // Instances overlap, as in the well-formed path: the 2nd "aa" of "aaa" starts at 2.
        (format!("=SUBSTITUTE(\"aaa\"&{h},\"aa\",\"x\",2)=\"ax\"&{h}"), "TRUE"),
        ("=SUBSTITUTE(\"aaa\",\"aa\",\"x\",2)".to_string(), "ax"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model._set("A1", &formula);
        model.evaluate();
        assert_eq!(model._get_text("A1"), expected, "{formula}");
    }
}
