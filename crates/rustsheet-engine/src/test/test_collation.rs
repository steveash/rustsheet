//! Excel's text order (`functions::collation`; probes/text-collation-order, T-0220).

use std::cmp::Ordering;

use crate::functions::util::collation::{cmp_text, cmp_text_excel, collation_key};
use crate::text::Text;

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

/// Two texts have the same collation key exactly when `cmp_text` finds them equal: the
/// exact-match index of XMATCH and XLOOKUP hashes the key (T-0448). Case, accents, expansions,
/// skipped hyphens and apostrophes, spaces, ASCII against non-ASCII, and lone surrogates.
#[test]
fn collation_key_equality_is_cmp_text_equality() {
    let mut texts: Vec<Text> = [
        "",
        "a",
        "A",
        "ab",
        "AB",
        "aB",
        "a-b",
        "a'b",
        "a\u{2013}b",
        "ab-",
        "-ab",
        "co-op",
        "coop",
        "Co-Op",
        "cafe",
        "café",
        "CAFÉ",
        "cafè",
        "ae",
        "æ",
        "Æ",
        "ss",
        "ß",
        "SS",
        "ij",
        "ĳ",
        "oe",
        "œ",
        "th",
        "þ",
        "u",
        "ü",
        "Ü",
        "ø",
        "ö",
        "d",
        "đ",
        "ð",
        "1",
        "¹",
        "01",
        " a",
        "a ",
        "a\u{a0}",
        "\u{1}a",
        "a\u{1}",
        "😀",
        "😀a",
        "x",
        "X",
        "ča",
        "Ča",
        "c\u{30c}a",
        "C\u{30c}-a",
        "č-a",
        "ca",
        "á",
        "a\u{301}",
        "ϊ",
        "ΐ",
        "ϊ\u{301}",
        "σ",
        "ς",
        "Σ",
        "е",
        "ё",
        "a\u{301}\u{30c}",
        "\u{301}a",
        "ª",
        "ª\u{301}",
        "à",
        "ǎ",
        "a\u{30c}",
        "a\u{342}\u{342}",
        "a\u{327}",
    ]
    .iter()
    .map(|s| Text::from(*s))
    .collect();
    texts.push(Text::from_utf16(vec![0xD800]));
    texts.push(Text::from_utf16(vec![0xD800, 0x61]));
    texts.push(Text::from_utf16(vec![0x61, 0xDC00]));
    for a in &texts {
        for b in &texts {
            assert_eq!(
                cmp_text(a, b) == Ordering::Equal,
                collation_key(a) == collation_key(b),
                "{a:?} vs {b:?}"
            );
        }
    }
}

/// Latin Extended-A, Greek and Cyrillic letters are their base letter plus an accent, and a
/// combining mark is the accent of the letter before it (probes/t3114-collation-beyond-latin1).
#[test]
fn letters_beyond_latin1_and_combining_marks() {
    assert_increasing(&["c", "ć", "č", "ca", "ča", "cb", "čb", "cz", "d"]);
    assert_increasing(&["a", "á", "ă", "ā", "ą", "b"]);
    assert_increasing(&["e", "é", "ė", "ě", "ē", "ę", "ea", "ěa", "eb"]);
    assert_increasing(&["o", "ö", "ő", "oa", "őa", "ob", "p"]);
    assert_increasing(&["u", "ū", "ů", "ų", "ű", "v", "z", "ź", "ż", "ž", "α"]);
    assert_increasing(&[
        "ι", "ί", "ϊ", "ΐ", "κ", "ω", "ώ", "а", "е", "ё", "ж", "и", "й", "к", "я",
    ]);
    for (a, b) in [
        ("a\u{301}", "á"),
        ("c\u{30c}", "č"),
        ("C\u{30c}", "č"),
        ("c\u{30c}a", "Ča"),
        ("Ő", "o\u{30b}"),
        ("Σ", "ς"),
        ("σ", "ς"),
        ("Ё", "ё"),
        ("Ά", "ά"),
        ("ϊ\u{301}", "ΐ"),
        // Pins an unprobed engine choice: a mark before a hyphen still belongs to its letter.
        ("C\u{30c}-a", "č-a"),
    ] {
        assert_eq!(cmp_text_excel(a, b), Ordering::Equal, "{a:?} = {b:?}");
    }
}

/// Accents are the Windows table's diacritic weights, and combining marks add theirs to the letter
/// before them in one wrapping byte (probes/t3154-collation-accents,
/// probes/t3154-collation-accent-weights, probes/t3154-collation-accent-pairs).
#[test]
fn accent_weights_add_up_and_wrap() {
    assert_increasing(&[
        "a", "á", "à", "â", "ä", "ǎ", "ă", "ā", "ã", "å", "ą", "ạ", "aa",
    ]);
    assert_increasing(&["e", "é", "è", "ė", "ê", "ë", "ě", "ē", "ę", "f"]);
    assert_increasing(&[
        "o", "ó", "ò", "ô", "ö", "ō", "õ", "ő", "ơ", "oa", "ơa", "ob",
    ]);
    assert_increasing(&["u", "ú", "ù", "û", "ü", "ū", "ů", "ų", "ű", "ư", "v"]);
    assert_increasing(&["s", "š", "ş", "ș", "sa", "șa", "sb"]);
    // Three dot-below marks wrap (2 + 3 * 86 = 260, so 4): between "a" and "á".
    assert_increasing(&["a", "a\u{323}\u{323}\u{323}", "á"]);
    assert_increasing(&["ª", "à", "ª\u{301}", "ȧ"]);
    for (a, b) in [
        ("a\u{301}\u{301}", "å"),
        ("a\u{301}\u{30c}", "a\u{30c}\u{301}"),
        ("á\u{30c}", "ǎ\u{301}"),
        ("à\u{323}", "a\u{323}\u{300}"),
        ("a\u{301}\u{300}", "ą"),
        ("a\u{342}\u{342}", "a\u{327}"),
        ("ǎ", "a\u{30c}"),
        ("Ș", "s\u{326}"),
        ("ț", "t\u{326}"),
        ("ư", "u\u{31b}"),
        ("ạ", "a\u{323}"),
        ("й", "и\u{306}"),
    ] {
        assert_eq!(cmp_text_excel(a, b), Ordering::Equal, "{a:?} = {b:?}");
        assert_eq!(collation_key(&Text::from(a)), collation_key(&Text::from(b)));
    }
    assert_ne!(cmp_text_excel("ș", "ş"), Ordering::Equal);
}

/// `decompose` binary-searches the table, so it must stay sorted by letter.
#[test]
fn decomposed_table_is_sorted() {
    use crate::functions::util::collation_tables::DECOMPOSED;
    assert!(DECOMPOSED.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

/// The exact and wildcard matches fold `ς` to `σ` on both sides
/// (probes/t3114-collation-beyond-latin1 Calc!H10; Rust's final-sigma rule lower-cases a word-final
/// `Σ` to `ς`).
#[test]
fn final_sigma_folds_in_exact_and_wildcard_matches() {
    use crate::functions::criteria::{folded_units, WildcardPattern};
    for pattern in ["ς", "σ", "Σ"] {
        let compiled = WildcardPattern::new(&Text::from(pattern));
        for text in ["ς", "σ", "Σ"] {
            assert!(
                compiled.matches(&Text::from(text)),
                "{pattern:?} matches {text:?}"
            );
        }
    }
    assert!(WildcardPattern::new(&Text::from("ς*")).matches(&Text::from("σx")));
    assert_eq!(
        folded_units(&Text::from("ΟΔΟΣ")),
        folded_units(&Text::from("οδοσ"))
    );
    assert_eq!(
        WildcardPattern::new(&Text::from("οδοσ")).literal_units(),
        Some(folded_units(&Text::from("ΟΔΟΣ")))
    );
}

/// The evaluated `Data` strings of the probes case `stem` (each a formula such as
/// `="a"&REPT(UNICHAR(769),22)`).
fn probe_data_strings(stem: &str) -> Vec<Text> {
    let path = format!(
        "{}/../../eval/suites/probes/cases/{stem}.case.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let case: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sheet = case["sheets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|sheet| sheet["name"] == "Data")
        .unwrap();
    let cells = sheet["cells"].as_object().unwrap();
    let mut model = crate::test::util::new_empty_model();
    for (cell, formula) in cells {
        model._set(cell, formula.as_str().unwrap());
    }
    model.evaluate();
    cells
        .keys()
        .map(|cell| Text::from(model._get_text(cell)))
        .collect()
}

/// The accent strings of the T-3154 and T-3201 probes and the kana strings of T-3275, on which
/// Excel's own comparison is not transitive (coercion.md "Not a total order"): `cmp_text`, which the sorts use, stays a total
/// order with `CollationKey` equality as its equality, and texts `cmp_text_pairwise` finds equal
/// share a key (what the criteria index relies on).
#[test]
fn cmp_text_is_a_total_order_on_the_accent_probes() {
    use crate::functions::util::collation::cmp_text_pairwise;
    for stem in [
        "t3154-collation-accent-weights",
        "t3154-collation-accent-pairs",
        "t3201-collation-long-mark-runs",
        "t3201-collation-wrapped-run-pairs",
        "t3275-collation-width-kana",
        "t3275-collation-width-kana-pairs",
    ] {
        let mut texts = probe_data_strings(stem);
        assert!(texts.len() > 20, "{stem}");
        texts.sort_by(cmp_text);
        for (i, a) in texts.iter().enumerate() {
            // Sorted, each text is equal to a run of the ones after it and below all the rest.
            let mut below = false;
            for b in &texts[i + 1..] {
                let order = cmp_text(a, b);
                assert_eq!(cmp_text(b, a), order.reverse(), "{stem}: {a:?} vs {b:?}");
                assert_eq!(
                    order,
                    if below {
                        Ordering::Less
                    } else {
                        order.min(Ordering::Equal)
                    },
                    "{stem}: {a:?} vs {b:?}"
                );
                below = order.is_lt();
                assert_eq!(
                    order.is_eq(),
                    collation_key(a) == collation_key(b),
                    "{stem}: {a:?} vs {b:?}"
                );
                if cmp_text_pairwise(a, b).is_eq() {
                    assert!(order.is_eq(), "{stem}: {a:?} vs {b:?}");
                }
            }
        }
    }
}

/// Excel's pairwise comparison: a text is below any text it is a proper prefix of, even when the
/// wrapped sums are equal or say otherwise (probes/t3181-collation-prefix-callers Calc!A1:A9,
/// probes/t3154-collation-accent-pairs Calc!S19, T18, W23, X22, AB28, AC27).
#[test]
fn pairwise_prefix_rule() {
    use crate::functions::util::collation::cmp_text_pairwise;
    let cmp = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    let circumflexes = format!("a{}", "\u{302}".repeat(16));
    assert_eq!(cmp("a", &circumflexes), Ordering::Less);
    assert_eq!(cmp(&circumflexes, "a"), Ordering::Greater);
    assert_eq!(
        cmp_text(&Text::from("a"), &Text::from(circumflexes.as_str())),
        Ordering::Equal
    );
    // U+0342 (141) < two U+0342 (28, = cedilla) < U+0342.
    assert_eq!(cmp("a\u{342}", "a\u{342}\u{342}"), Ordering::Less);
    assert_eq!(cmp("a\u{342}\u{342}", "a\u{327}"), Ordering::Equal);
    assert_eq!(cmp("a\u{327}", "a\u{342}"), Ordering::Less);
    // U+0323 (86) < three U+0323 (4) < á (14) < U+0323.
    assert_eq!(cmp("a\u{323}", "a\u{323}\u{323}\u{323}"), Ordering::Less);
    assert_eq!(cmp("a\u{323}\u{323}\u{323}", "á"), Ordering::Less);
    assert_eq!(cmp("á", "a\u{323}"), Ordering::Less);
    // Precomposed á is one unit: it equals a plus U+0301.
    assert_eq!(cmp("á", "a\u{301}"), Ordering::Equal);
}

/// Full-width forms, kana, voicing and the long-vowel and iteration marks
/// (probes/t3275-collation-width-kana, probes/t3275-collation-width-kana-pairs; coercion.md
/// "Full-width and half-width forms, kana, voicing and the long-vowel mark").
#[test]
fn width_and_kana_tiebreaks() {
    use crate::functions::util::collation::cmp_text_pairwise;
    assert_increasing(&["0", "０", "1", "１", "１a", "1b"]);
    assert_increasing(&[" ", "\u{3000}", "!", "！", "#"]);
    assert_increasing(&[
        "A", "Ａ", "á", "aa", "Ａa", "aá", "ab", "a＇b", "a-b", "a－b", "AＢ", "Ａb",
    ]);
    assert_increasing(&["", "'", "＇", "-", "－", "゛", " "]);
    assert_increasing(&[
        "я", "ぁ", "ｱ", "ア", "あ", "ｱあ", "アあ", "あア", "ああ", "ぁい", "い",
    ]);
    assert_increasing(&["かぁ", "カア", "かあ", "カー", "かー", "があ", "かい"]);
    assert_increasing(&["か", "ｶﾞ", "ガ", "が", "ッ", "っ", "つ", "は", "ば", "ぱ"]);
    assert_increasing(&["かか", "かゝ", "かが", "かゞ", "き", "きい", "きー"]);
    assert_increasing(&["一", "人", "人々", "人人", "ー", "ーa"]);
    for (left, right) in [
        ("Ａ", "ａ"),
        ("か\u{3099}", "が"),
        ("は\u{309a}", "ぱ"),
        ("ー", "ゝ"),
        ("ゞ", "々"),
    ] {
        assert_eq!(
            cmp_text_excel(left, right),
            Ordering::Equal,
            "{left:?} = {right:?}"
        );
    }
    // `"aー"` sorts as `"aa"`, but Excel's `<` compares its `ー` alone where the texts differ.
    let cmp = |a: &str, b: &str| cmp_text_pairwise(&Text::from(a), &Text::from(b));
    assert_eq!(cmp_text_excel("aー", "aa"), Ordering::Equal);
    assert_eq!(cmp("aー", "aa"), Ordering::Greater);
    assert_eq!(cmp("aー", "ab"), Ordering::Greater);
    assert_eq!(cmp("aー", "AＢ"), Ordering::Greater);
    assert_eq!(cmp("aー", "Ａa"), Ordering::Less);
    assert_eq!(cmp("aー", "Ａb"), Ordering::Less);
}

/// Exact MATCH folds `ー` after a letter to that letter and a leading iteration mark to `ー`
/// (probes/t3275-collation-width-kana Calc!C127:C129, C136), and does not decompose voicing.
#[test]
fn exact_match_fold_of_the_marks() {
    use crate::functions::util::collation::fold_exact_lookup;
    let fold = |text: &str| {
        let units: Vec<u16> = text.encode_utf16().collect();
        String::from_utf16_lossy(&fold_exact_lookup(&units))
    };
    assert_eq!(fold("aー"), "aa");
    assert_eq!(fold("ゝ"), "ー");
    assert_eq!(fold("々"), "ー");
    assert_eq!(fold("かー"), "かー");
    assert_eq!(fold("かゝ"), "かゝ");
    assert_eq!(fold("a々"), "a々");
    assert_ne!(fold("か\u{3099}"), fold("が"));
    assert_eq!(fold_exact_lookup(&[0xD800, 0x30FC]), vec![0xD800, 0x30FC]);
}
