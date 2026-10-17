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
