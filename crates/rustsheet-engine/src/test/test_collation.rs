//! Excel's text order (`functions::collation`; probes/text-collation-order, T-0220).

use std::cmp::Ordering;

use crate::functions::util::collation::cmp_text_excel;

/// Asserts `texts` are strictly increasing in Excel's order.
fn assert_increasing(texts: &[&str]) {
    for pair in texts.windows(2) {
        assert_eq!(
            cmp_text_excel(pair[0], pair[1]),
            Ordering::Less,
            "{:?} < {:?}",
            pair[0],
            pair[1]
        );
        assert_eq!(cmp_text_excel(pair[1], pair[0]), Ordering::Greater);
    }
}

#[test]
fn punctuation_digits_letters() {
    assert_increasing(&[
        "", "'", "''", "-", "--", " ", "  ", "\u{a0}", "!", "\"", "#", "$", "%", "&", "(", ")",
        "*", ",", ".", "/", ":", ";", "?", "@", "[", "\\", "]", "^", "_", "`", "{", "|", "}", "~",
        "+", "<", "=", ">", "0", "1", "9", "a", "b", "z",
    ]);
}

#[test]
fn hyphen_and_apostrophe_break_ties_only() {
    assert_increasing(&["1", "'1", "-1", "2"]);
    assert_increasing(&["a", "a'", "a-", "'a", "-a", "aa"]);
    assert_increasing(&["ab", "a'b", "a-b", "æ", "b"]);
    assert_increasing(&["co op", "coop", "co-op"]);
    // gen-m2/96a0952412cefa6c A284: punctuation before digits.
    assert_increasing(&["?b", "1.5"]);
}

#[test]
fn accents_are_a_later_tiebreak() {
    assert_increasing(&["a", "á", "à", "â", "ä", "å", "aa"]);
    assert_increasing(&["e", "é", "f"]);
    assert_increasing(&["o", "ö", "ø", "p"]);
    assert_increasing(&["resume", "résumé"]);
    assert_increasing(&["y", "ÿ", "z"]);
    // gen-m2/2cfe279fe77a000d A318: "ü" sorts with "u", before "x".
    assert_increasing(&["u", "ü", "x"]);
}

#[test]
fn case_and_expansions_are_equal() {
    for (left, right) in [
        ("a", "A"),
        ("Resume", "resume"),
        ("Résumé", "résumé"),
        ("À", "à"),
        ("æ", "ae"),
        ("Æ", "AE"),
        ("ß", "ss"),
        ("ß", "SS"),
    ] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
    assert_ne!(cmp_text_excel("a-b", "ab"), Ordering::Equal);
    assert_ne!(cmp_text_excel("\u{a0}1", " 1"), Ordering::Equal);
}

#[test]
fn other_characters_keep_code_point_order_after_letters() {
    assert_increasing(&["\u{1}", " ", "z", "α", "β", "ω", "😀"]);
}

/// probes/text-collation-unprobed (T-1590, excel-graph): the whole observed order, lowest first.
#[test]
fn t1590_probed_characters() {
    assert_increasing(&[
        "", "\u{1}", "\u{1f}", "\u{7f}", "–", "—", "\u{a0}", "\t", "\n", "\r", "!", "~", "¡", "¦",
        "¨", "¯", "´", "¸", "¿", "‘", "’", "“", "”", "¢", "£", "¤", "¥", "€", "+", "±", "«", "»",
        "×", "÷", "§", "©", "¬", "®", "°", "µ", "…", "•", "0", "¼", "½", "¾", "¹", "²", "³", "9",
        "a", "ª", "á", "ä", "ã", "å", "a’b", "ab", "a'b", "a-b", "a–b", "a—b", "c", "ç", "d", "đ",
        "ð", "i", "ı", "ij", "l", "ł", "n", "ñ", "o", "º", "ö", "ø", "œ", "s", "t", "þ", "u", "z",
        "α", "β", "ω",
    ]);
    for (left, right) in [("ĳ", "ij"), ("Ĳ", "ij"), ("Z", "z")] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
}

#[test]
fn ascii_fast_path_agrees_with_the_general_path() {
    // A shared non-ASCII first character forces the general path without changing any pass:
    // equal letter and accent weights, and every hyphen/apostrophe position shifts by one.
    let samples = [
        "", "'", "-", " ", "!", "+", "0", "9", "a", "A", "a-b", "a'b", "ab", "co-op", "coop", "~",
        "-1", "'1", "1", "z", "Z_", "a\u{1}", "\u{7f}", "\t", "a\r",
    ];
    for left in samples {
        for right in samples {
            let prefixed = |s: &str| format!("\u{3b1}{s}");
            assert_eq!(
                cmp_text_excel(left, right),
                cmp_text_excel(&prefixed(left), &prefixed(right)),
                "{left:?} vs {right:?}"
            );
        }
    }
}
