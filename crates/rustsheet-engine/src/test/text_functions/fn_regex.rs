use crate::test::util::new_empty_model;

// ── REGEXTEST ─────────────────────────────────────────────────────────────────

#[test]
fn test_regextest_basic_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"hello world\", \"world\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
}

#[test]
fn test_regextest_no_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"hello world\", \"xyz\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
}

#[test]
fn test_regextest_anchored() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"abc123\", \"^[a-z]+$\")");
    model._set("A2", "=REGEXTEST(\"abc123\", \"^[a-z0-9]+$\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("A2"), "TRUE");
}

#[test]
fn test_regextest_invalid_regex() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"text\", \"[\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn test_regextest_case_sensitive() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"Hello\", \"hello\")");
    model._set("A2", "=REGEXTEST(\"Hello\", \"Hello\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("A2"), "TRUE");
}

#[test]
fn test_regextest_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXTEST(\"text\")");
    model._set("A2", "=REGEXTEST(\"text\", \".\", \"extra\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

// ── REGEXEXTRACT ──────────────────────────────────────────────────────────────

#[test]
fn test_regexextract_full_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"foo 42 bar\", \"[0-9]+\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "42");
}

#[test]
fn test_regexextract_capture_group() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=REGEXEXTRACT(\"2024-01-15\", \"(\\d{4})-\\d{2}-\\d{2}\")",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2024");
}

#[test]
fn test_regexextract_no_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"hello\", \"[0-9]+\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_regexextract_invalid_regex() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"text\", \"[\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn test_regexextract_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"text\")");
    model._set("A2", "=REGEXEXTRACT(\"text\", \".\", \"0\", \"extra\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn test_regexextract_all_matches() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"LonelyPlanet\", \"[A-Z][a-z]+\", 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "Lonely");
    assert_eq!(model._get_text("B1"), "Planet");
}

#[test]
fn test_regexextract_all_matches_no_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"nouppercase\", \"[A-Z][a-z]+\", 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

// ── REGEXREPLACE ──────────────────────────────────────────────────────────────

#[test]
fn test_regexreplace_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXREPLACE(\"hello world\", \"world\", \"Rust\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hello Rust");
}

#[test]
fn test_regexreplace_all_occurrences() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXREPLACE(\"aabbcc\", \"[a-c]\", \"x\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "xxxxxx");
}

#[test]
fn test_regexreplace_no_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXREPLACE(\"hello\", \"[0-9]+\", \"NUM\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hello");
}

#[test]
fn test_regexreplace_digits() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=REGEXREPLACE(\"abc 123 def 456\", \"[0-9]+\", \"N\")",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "abc N def N");
}

#[test]
fn test_regexreplace_invalid_regex() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXREPLACE(\"text\", \"[\", \"x\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

// T-1688: the replacement is PCRE2's extended syntax (text.md "REGEXREPLACE replacement syntax",
// probes/regex-pcre2-features).
#[test]
fn regexreplace_replacement_syntax() {
    let mut model = new_empty_model();
    let cases = [
        (r#""ab","(a)","\1\1""#, "aab"),
        (r#""ab","(a)","[$1a]""#, "[aa]b"),
        (r#""ab","(a)","[\$1|\\|$$]""#, r"[$1|\|$]b"),
        (r#""ab","(?<x>a)","${x}$x${1}""#, "aaab"),
        (r#""ab","a","[\$\t]""#, "[$\t]b"),
        (r#""zz","(a)","\1""#, "zz"),
        (r#""ab","(a)","$2""#, "#VALUE!"),
        (r#""ab","(a)","[\10]""#, "#VALUE!"),
        (r#""ab","(a)","${y}""#, "#VALUE!"),
        (r#""ab","(a)","$""#, "#VALUE!"),
        (r#""ab","(a)","x\""#, "#VALUE!"),
        (r#""ab","(a)","\0""#, "#VALUE!"),
        (r#""ab","(a)","\q""#, "#VALUE!"),
    ];
    for (row, (args, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), &format!("=REGEXREPLACE({args})"));
    }
    model.evaluate();
    for (row, (args, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{args}"
        );
    }
}

#[test]
fn test_regexreplace_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXREPLACE(\"text\", \".\")");
    model._set("A2", "=REGEXREPLACE(\"text\", \".\", \"x\", \"extra\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

// T-1272 performance review: a call compiles each distinct pattern once, keeps it and never
// evicts, so 600 patterns that cycle compile 600 times; 5,000 distinct ones are past the 2,048
// small patterns a call may compile, `#ERROR!` (DEVIATION, decision 0014).
#[test]
fn regex_compiles_are_cached_and_budgeted() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ROWS(REGEXTEST(\"a\",\"a\"&MOD(SEQUENCE(1000),600)))",
    );
    model._set(
        "A2",
        "=SUM(--REGEXTEST(\"a\"&MOD(SEQUENCE(1000),600),\"a\"&MOD(SEQUENCE(1000),600)))",
    );
    model._set("A3", "=ROWS(REGEXTEST(\"a\",\"a\"&SEQUENCE(5000)))");
    model._set("A4", "=REGEXTEST(\"ab\",\"\\w{2}\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1000");
    assert_eq!(model._get_text("A2"), *"1000");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    // `\w{2}` is too big for the smallest size limit and compiles under the next one.
    assert_eq!(model._get_text("A4"), *"TRUE");
}

// A lifted REGEXEXTRACT element with return_all keeps the first match only.
#[test]
fn lifted_regexextract_return_all_keeps_the_first_match() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT({\"a1b2\";\"xyz\"},\"[0-9]\",1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#N/A");
}

// T-1272 performance review: patterns too big for the smaller size limits that cycle through a
// lifted call compile once each instead of thrashing a cache.
#[test]
fn cycling_patterns_compile_once() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=SUM(--REGEXTEST(\"abcdefghijklmnop\",\"\\w{\"&(2+MOD(SEQUENCE(400),15))&\"}\"))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"400");
}

// T-1817 (folded into T-1272): REGEXREPLACE knows its result length before it builds it. Over
// 32,767 characters it is `#VALUE!` as SUBSTITUTE is (UNVERIFIED for REGEXREPLACE); an empty
// pattern over 32,767 characters with a 10-character replacement is refused without building
// its 360,000 characters.
#[test]
fn regexreplace_result_is_bounded_before_it_is_built() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=LEN(REGEXREPLACE(REPT(\"a\",32767),\"\",\"bbbbbbbbbb\"))",
    );
    model._set("A2", "=LEN(REGEXREPLACE(REPT(\"a\",32767),\"a\",\"bb\"))");
    model._set("A3", "=LEN(REGEXREPLACE(REPT(\"a\",16383),\"a\",\"bb\"))");
    model._set(
        "A4",
        "=LEN(REGEXREPLACE(REPT(\"ab\",16000),\"(a)(b)\",\"$2$1$1\"))",
    );
    model._set(
        "A5",
        "=LEN(REGEXREPLACE(REPT(\"ab\",10000),\"(a)(b)\",\"$2$1$1\"))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"32766");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"30000");
}

// T-1272 performance review: iterating matches on the PikeVM (a `\b` pattern over non-ASCII
// text, where the lazy DFA quits) charges each successive search the rest of the text, so a few
// KB with many matches is `#ERROR!` (DEVIATION, decision 0014; T-1829 charges the work done and
// updates this test). ASCII text stays on the lazy DFA.
#[test]
fn pike_iteration_is_charged_by_rest_of_text() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=COLUMNS(REGEXEXTRACT(REPT(\"naïve \",300),\"\\b\\w+\\b\",1))",
    );
    model._set(
        "A2",
        "=COLUMNS(REGEXEXTRACT(REPT(\"naïve \",500),\"\\b\\w+\\b\",1))",
    );
    model._set(
        "A3",
        "=LEN(REGEXREPLACE(REPT(\"cafe foo bar \",2500),\"\\bfoo\\b\",\"x\"))",
    );
    model._set(
        "A4",
        "=LEN(REGEXREPLACE(REPT(\"café foo bar \",2500),\"\\bfoo\\b\",\"x\"))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"300");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"27500");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}
