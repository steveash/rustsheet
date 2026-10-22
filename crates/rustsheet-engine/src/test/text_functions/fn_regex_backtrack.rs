//! REGEX* patterns only the backtracker runs (T-1900, SPEC-0013 section 3): backreferences,
//! atomic and possessive groups, `\G`.

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
