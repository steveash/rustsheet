//! REGEX* patterns only the backtracker runs (T-1900, SPEC-0013 section 3): backreferences,
//! atomic and possessive groups, `\G`, `\R` and `\X`.

use crate::test::util::new_empty_model;

/// The displayed value of `formula`, evaluated alone.
fn value(formula: &str) -> String {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

#[test]
fn backreferences_match_the_group_again() {
    // probes/regex-pcre2-features C1-C5 and probes/t1626-regextest C29 (excel-graph).
    assert_eq!(value(r#"=REGEXTEST("abab","^(ab)\1$")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("abac","^(ab)\1$")"#), "FALSE");
    assert_eq!(
        value(r#"=REGEXEXTRACT("abcabc","(?<n>abc)\k<n>")"#),
        "abcabc"
    );
    assert_eq!(value(r#"=REGEXEXTRACT("xaay","(a)\g1")"#), "aa");
    assert_eq!(value(r#"=REGEXEXTRACT("xAay","(a)\1",0,1)"#), "Aa");
    assert_eq!(value(r#"=REGEXTEST("hello","(\w)\1")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("helo","(\w)\1")"#), "FALSE");
    // `(?i)` scopes the backreference's folding as it does a literal's.
    assert_eq!(value(r#"=REGEXTEST("aA","(a)(?i)\1")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("aA","(a)\1")"#), "FALSE");
    // A group that did not take part in the match never matches again (PCRE2's default).
    assert_eq!(value(r#"=REGEXTEST("b","(a)?b\1")"#), "FALSE");
}

#[test]
fn a_group_changes_its_capture_only_when_it_closes() {
    // PCRE2 (and Perl): inside its own repeated group a backreference sees the last completed
    // iteration, not the one in progress (Excel-fidelity review of T-1900; unprobed in Excel).
    assert_eq!(value(r#"=REGEXTEST("abab","^(a|b\1)+$")"#), "FALSE");
    assert_eq!(value(r#"=REGEXEXTRACT("abab","^(a|b\1)+")"#), "aba");
    assert_eq!(value(r#"=REGEXTEST("aba","^(a|b\1)+$")"#), "TRUE");
}

#[test]
fn an_occurrence_keeps_the_match_and_groups_the_iteration_found() {
    // `\G` stands where the search that found the match started, with or without an occurrence.
    assert_eq!(value(r#"=REGEXREPLACE("xab","\Gab|a","[$0]")"#), "x[a]b");
    assert_eq!(value(r#"=REGEXREPLACE("xab","\Gab|a","[$0]",1)"#), "x[a]b");
    assert_eq!(
        value(r#"=REGEXREPLACE("ab ab","(a)(b)\1?","<$2>",2)"#),
        "ab <b>"
    );
}

#[test]
fn backreferences_iterate_with_their_captures() {
    assert_eq!(
        value(r#"=REGEXREPLACE("aa bb cd","(\w)\1","<$1>")"#),
        "<a> <b> cd"
    );
    assert_eq!(
        value(r#"=TEXTJOIN("|",FALSE,REGEXEXTRACT("xx-yy-z","(\w)\1",1))"#),
        "xx|yy"
    );
    assert_eq!(
        value(r#"=TEXTJOIN("|",FALSE,REGEXEXTRACT("abab","(a)(b)\1\2",2))"#),
        "a|b"
    );
}

#[test]
fn atomic_and_possessive_groups_give_back_nothing() {
    // probes/regex-pcre2-features B29 and C11: `a++a` cannot match.
    assert_eq!(value(r#"=ERROR.TYPE(REGEXEXTRACT("aaa","a++a"))"#), "7");
    assert_eq!(value(r#"=ISERROR(REGEXEXTRACT("aaa","a++a"))"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("aaa","(?>a*)a")"#), "FALSE");
    assert_eq!(value(r#"=REGEXEXTRACT("aaab","a*+b")"#), "aaab");
    assert_eq!(value(r#"=REGEXEXTRACT("abc","(?>ab|a)c")"#), "abc");
    assert_eq!(value(r#"=REGEXTEST("ac","^(?>ab|a)c")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("abc","^(?>a|ab)c")"#), "FALSE");
    // A capture made inside an atomic group is undone when the match backtracks past it: group
    // 1 is unset (`#N/A`, as on the automata) in the match the second alternative makes.
    assert_eq!(
        value(r#"=ISNA(INDEX(REGEXEXTRACT("aay","(?:(?>(a))z|a(a)?)y",2),1))"#),
        "TRUE"
    );
    assert_eq!(
        value(r#"=INDEX(REGEXEXTRACT("aay","(?:(?>(a))z|a(a)?)y",2),2)"#),
        "a"
    );
}

#[test]
fn empty_iterations_end_a_loop() {
    // PCRE2's empty-loop rule: an iteration that matched the empty string ends the loop.
    assert_eq!(value(r#"=REGEXEXTRACT("aab","(?:a*)*+b")"#), "aab");
    assert_eq!(value(r#"=REGEXEXTRACT("b","(a?)*+b")"#), "b");
    assert_eq!(value(r#"=REGEXEXTRACT("aab","(?>(a|)*)b")"#), "aab");
}

#[test]
fn search_start_anchors_where_the_search_starts() {
    assert_eq!(value(r#"=REGEXTEST("xa","\Ga")"#), "FALSE");
    assert_eq!(value(r#"=REGEXTEST("ax","\Ga")"#), "TRUE");
    assert_eq!(value(r#"=REGEXREPLACE("aab","\Ga","-")"#), "--b");
}

/// Catastrophic patterns on the backtracker (each made non-regular with `\G` or a
/// backreference) end in `#ERROR!` (the call's work spent) or an answer, without a stack
/// overflow, within a wall-clock bound. The bound is for a release build; a debug build is about
/// ten times slower.
#[test]
fn catastrophic_patterns_stop() {
    let bound = if cfg!(debug_assertions) { 120 } else { 10 };
    for pattern in [r"\G(a|a)*b", r"\G(a*)*b", r"\G((a+)+)+b", r"(.*)\1x"] {
        let start = std::time::Instant::now();
        let result = value(&format!(r#"=REGEXTEST(REPT("a",32767),"{pattern}")"#));
        assert!(
            result == "#ERROR!" || result == "FALSE",
            "{pattern}: {result}"
        );
        assert!(start.elapsed().as_secs() < bound, "{pattern}");
    }
}

/// Repeats of a body that compiles to nothing still count against the compile limit (security
/// review of T-1900): each nesting multiplies the copies, which once cost nothing.
#[test]
fn empty_repeat_bodies_are_charged_at_compile() {
    for pattern in [
        r"(?:(?:){65535}){65535}(a)\1",
        r"(?:(?:){0,65535}){0,65535}(a)\1",
        r"(?:(?:(?:){65535}){65535}){65535}(a)\1",
    ] {
        let start = std::time::Instant::now();
        let result = value(&format!(r#"=REGEXTEST("aa","{pattern}")"#));
        assert!(
            result == "#VALUE!" || result == "#ERROR!",
            "{pattern}: {result}"
        );
        assert!(start.elapsed().as_secs() < 10, "{pattern}");
    }
}

#[test]
fn line_breaks_and_grapheme_clusters_are_atomic() {
    // probes/t2287-regex-r-x-spans (excel-graph): `\r\n` is one `\R` and one `\X`, and neither
    // gives a character back.
    let cr_lf = r#"CHAR(13)&CHAR(10)"#;
    assert_eq!(value(&format!(r#"=REGEXTEST({cr_lf},"^\R$")"#)), "TRUE");
    assert_eq!(value(&format!(r#"=REGEXTEST({cr_lf},"^\R\n$")"#)), "FALSE");
    assert_eq!(value(&format!(r#"=REGEXTEST({cr_lf},"^\X\n$")"#)), "FALSE");
    assert_eq!(value(r#"=REGEXTEST(UNICHAR(8232),"\R")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("a","[\R]")"#), "#VALUE!");
    assert_eq!(value(r#"=REGEXTEST("e"&UNICHAR(769),"^\X$")"#), "TRUE");
    assert_eq!(value(r#"=REGEXTEST("e"&UNICHAR(769),"^\X.$")"#), "FALSE");
    assert_eq!(
        value(r#"=REGEXREPLACE("e"&UNICHAR(769)&"a","\X","[$0]")"#),
        "[e\u{301}][a]"
    );
}

#[test]
fn grapheme_clusters_follow_pcre2s_join_table() {
    // probes/t2287-regex-r-x-spans: the lengths of each `\X` in the text.
    let lengths = |text: &str| {
        value(&format!(
            r#"=TEXTJOIN(",",,LEN(REGEXEXTRACT({text},"\X",1)))"#
        ))
    };
    // A ZWJ family emoji and a flag are one cluster each; three indicators are a pair and one.
    assert_eq!(
        lengths("UNICHAR(128104)&UNICHAR(8205)&UNICHAR(128105)&UNICHAR(8205)&UNICHAR(128103)"),
        "8"
    );
    assert_eq!(
        lengths("UNICHAR(127482)&UNICHAR(127480)&UNICHAR(127482)"),
        "4,2"
    );
    // No GB9c: a Devanagari conjunct is two clusters (A20).
    assert_eq!(lengths("UNICHAR(2325)&UNICHAR(2381)&UNICHAR(2359)"), "2,1");
    // PCRE2's quirks: nothing but an indicator joins an indicator, and a Prepend does not join
    // an emoji (B3, B4), though it joins an indicator (B5).
    assert_eq!(lengths("UNICHAR(127482)&UNICHAR(769)"), "2,1");
    assert_eq!(lengths("UNICHAR(1536)&UNICHAR(128512)"), "1,2");
    assert_eq!(lengths("UNICHAR(1536)&UNICHAR(127482)"), "3");
    // A ZWJ joins an emoji only after an emoji and its extenders (A24, B17, B19).
    assert_eq!(lengths(r#""a"&UNICHAR(8205)&UNICHAR(128104)"#), "2,2");
    assert_eq!(
        lengths("UNICHAR(128104)&UNICHAR(769)&UNICHAR(8205)&UNICHAR(128105)"),
        "6"
    );
    assert_eq!(
        lengths("UNICHAR(128512)&UNICHAR(2307)&UNICHAR(8205)&UNICHAR(128512)"),
        "4,2"
    );
}

/// `\X` charges each character it takes like any other step (directive 5), and a text of the
/// longest length a cell holds still fits in a call's budget.
#[test]
fn grapheme_clusters_over_a_long_text_fit_the_budget() {
    assert_eq!(
        value(r#"=LEN(REGEXEXTRACT(REPT("a",32767),"\X+"))"#),
        "32767"
    );
    assert_eq!(
        value(r#"=LEN(REGEXEXTRACT(REPT("e"&UNICHAR(769),16383),"\X+"))"#),
        "32766"
    );
    // One cluster of 32,767 characters, read again from every start: an answer or the call's
    // work spent, quickly.
    let bound = if cfg!(debug_assertions) { 120 } else { 10 };
    let start = std::time::Instant::now();
    let result = value(r#"=REGEXTEST("a"&REPT(UNICHAR(769),32766),"\Xb")"#);
    assert!(result == "#ERROR!" || result == "FALSE", "{result}");
    assert!(start.elapsed().as_secs() < bound);
}
