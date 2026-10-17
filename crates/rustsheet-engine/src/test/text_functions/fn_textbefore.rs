#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Brian Wiles");
    model._set("A2", "Jeden,dwa,trzy,cztery");

    model._set("B1", "=TEXTAFTER(A1, \" \")");
    model._set("B2", "=TEXTAFTER(A2, \",\")");
    model._set("C2", "=TEXTAFTER(A2, \",\", 2)");

    model._set("H1", "=TEXTBEFORE(A1, \" \")");
    model._set("H2", "=TEXTBEFORE(A2, \",\")");
    model._set("I2", "=_xlfn.TEXTBEFORE(A2, \",\", 2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Wiles");
    assert_eq!(model._get_text("B2"), *"dwa,trzy,cztery");
    assert_eq!(model._get_text("C2"), *"trzy,cztery");

    assert_eq!(model._get_text("H1"), *"Brian");
    assert_eq!(model._get_text("H2"), *"Jeden");
    assert_eq!(model._get_text("I2"), *"Jeden,dwa");
    assert_eq!(model._get_formula("I2"), *"=TEXTBEFORE(A2,\",\",2)");
}

// T-1796: match_mode 1 once located the delimiter in `to_lowercase()` of the text and sliced the
// original at those byte offsets; İ lowercases to two chars, so the offsets ran past the end.
#[test]
fn test_textbefore_textafter_case_insensitive_non_ascii_length_change() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTBEFORE(\"İİİ,a\",\",\",1,1)");
    model._set("A2", "=TEXTAFTER(\"İİİ,a\",\",\",1,1)");
    model._set("A3", "=TEXTBEFORE(\"İİİ,a,b\",\",\",-1,1)");
    model._set("A4", "=TEXTAFTER(\"İİİ,a,b\",\",\",-1,1)");
    // Kelvin sign (3 bytes) lowercases to ASCII k (1 byte), shifting offsets the other way.
    model._set("A5", "=TEXTBEFORE(\"\u{212A}\u{212A},b\",\",\",1,1)");
    model._set("A6", "=TEXTAFTER(\"\u{212A}\u{212A},b\",\",\",-1,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "İİİ");
    assert_eq!(model._get_text("A2"), "a");
    assert_eq!(model._get_text("A3"), "İİİ,a");
    assert_eq!(model._get_text("A4"), "b");
    assert_eq!(model._get_text("A5"), "\u{212A}\u{212A}");
    assert_eq!(model._get_text("A6"), "b");
}

// Excel (probes/textbefore-casefold-length): same folding as TEXTSPLIT: İ matches "i" (either
// way round), the Kelvin sign does not match "k", ß / ẞ do not match "ss", Ä matches ä.
#[test]
fn test_textbefore_textafter_case_insensitive_matches_excel_folding() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTBEFORE(\"aİbİc\",\"i\",1,1)");
    model._set("A2", "=TEXTAFTER(\"aİbİc\",\"i\",1,1)");
    model._set("A3", "=TEXTBEFORE(\"aibic\",\"İ\",1,1)");
    model._set("A4", "=TEXTBEFORE(\"a\u{212A}bkc\",\"k\",1,1)");
    model._set("A5", "=TEXTBEFORE(\"akbkc\",\"\u{212A}\",1,1)");
    model._set("A6", "=TEXTBEFORE(\"aßbSSc\",\"ss\",1,1)");
    model._set("A7", "=TEXTBEFORE(\"assbẞc\",\"ß\",1,1)");
    model._set("A8", "=TEXTBEFORE(\"ÄxäyÄ\",\"ä\",2,1)");
    model._set("A9", "=TEXTAFTER(\"ÄxäyÄ\",\"ä\",2,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a");
    assert_eq!(model._get_text("A2"), "bİc");
    assert_eq!(model._get_text("A3"), "a");
    assert_eq!(model._get_text("A4"), "a\u{212A}b");
    assert_eq!(model._get_text("A5"), "#N/A");
    assert_eq!(model._get_text("A6"), "aßb");
    assert_eq!(model._get_text("A7"), "#N/A");
    assert_eq!(model._get_text("A8"), "Äx");
    assert_eq!(model._get_text("A9"), "yÄ");
}

// Excel (probes/textbefore-casefold-length A17-A25): sigma folds by position, not to a single
// letter: a Σ followed by a letter is σ, any other Σ is ς, and ς matches only ς.
#[test]
fn test_textbefore_textafter_case_insensitive_final_sigma() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTBEFORE(\"aΟΣb\",\"ος\",1,1)");
    model._set("A2", "=TEXTAFTER(\"aΟΣb\",\"ς\",1,1)");
    model._set("A3", "=TEXTBEFORE(\"aοςb\",\"Σ\",1,1)");
    model._set("A4", "=TEXTBEFORE(\"aοςb\",\"ς\",1,1)");
    model._set("A5", "=TEXTBEFORE(\"aοσb\",\"ς\",1,1)");
    model._set("A6", "=TEXTBEFORE(\"aοσb\",\"Σ\",1,1)");
    model._set("A7", "=TEXTBEFORE(\"aοςb\",\"σ\",1,1)");
    model._set("A8", "=TEXTBEFORE(\"aΟΣb\",\"σ\",1,1)");
    model._set("A9", "=TEXTBEFORE(\"aΟΣ\",\"ς\",1,1)");
    model.evaluate();
    for cell in ["A1", "A2", "A5", "A6", "A7"] {
        assert_eq!(model._get_text(cell), "#N/A", "{cell}");
    }
    assert_eq!(model._get_text("A3"), "aο");
    assert_eq!(model._get_text("A4"), "aο");
    assert_eq!(model._get_text("A8"), "aΟ");
    assert_eq!(model._get_text("A9"), "aΟ");
}

// A huge negative instance number saturates to i32::MIN in the caller; negating it must not
// overflow (T-1796 security review). It behaves like the largest representable negative count.
#[test]
fn test_textbefore_textafter_huge_negative_instance_does_not_overflow() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTBEFORE(\"a,b\",\",\",-1E10)");
    model._set("A2", "=TEXTBEFORE(\"a,b\",\",\",-1E10,1)");
    model._set("A3", "=TEXTAFTER(\"a,b\",\",\",-1E10)");
    model._set("A4", "=TEXTAFTER(\"a,b\",\",\",-1E10,1)");
    model._set("B1", "=TEXTBEFORE(\"a,b\",\",\",-2147483647)");
    model._set("B2", "=TEXTBEFORE(\"a,b\",\",\",-2147483647,1)");
    model._set("B3", "=TEXTAFTER(\"a,b\",\",\",-2147483647)");
    model._set("B4", "=TEXTAFTER(\"a,b\",\",\",-2147483647,1)");
    model.evaluate();
    for row in 1..=4 {
        assert_eq!(
            model._get_text(&format!("A{row}")),
            model._get_text(&format!("B{row}")),
            "row {row}"
        );
    }
}

// T-1272: the text lifts over a range, the delimiter is a list (probes/m7-argument-modes F13-K13).
#[test]
fn text_lifts_and_delimiters_are_a_list() {
    let mut model = new_empty_model();
    model._set("A1", "a,b");
    model._set("A2", "c-d");
    model._set("A3", "e,f");
    model._set("C1", "=TEXTBEFORE(A1:A3,{\",\";\"-\"})");
    model._set("D1", "=TEXTAFTER(A1:A3,{\",\";\"-\"})");
    model._set("E1", "=TEXTBEFORE(\"a,b,c\",\",\",{1;2})");
    model._set("F1", "=TEXTSPLIT(A1:A3,\",\")");
    model._set("G1", "=REGEXTEST(A1:A3,\"a\")");
    model._set("H1", "=REGEXEXTRACT(\"a1b2\",{\"[0-9]\";\"[a-z]\"})");
    model._set("I1", "=REGEXREPLACE(A1:A3,\",\",\"+\")");
    model.evaluate();

    let column = |letter: char, rows: i32| -> Vec<String> {
        let c = i32::from(letter as u8 - b'A') + 1;
        (1..=rows).map(|r| model._get_text_at(0, r, c)).collect()
    };
    assert_eq!(column('C', 3), ["a", "c", "e"]);
    assert_eq!(column('D', 3), ["b", "d", "f"]);
    assert_eq!(column('E', 2), ["a", "a,b"]);
    assert_eq!(column('F', 3), ["a", "c-d", "e"]);
    assert_eq!(column('G', 3), ["TRUE", "FALSE", "FALSE"]);
    assert_eq!(column('H', 2), ["1", "a"]);
    assert_eq!(column('I', 3), ["a+b", "c-d", "e+f"]);
}
