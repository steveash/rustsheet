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
fn unprobed_characters_keep_code_point_order_after_letters() {
    assert_increasing(&["\u{1}", " ", "z", "þ", "α", "😀"]);
}
