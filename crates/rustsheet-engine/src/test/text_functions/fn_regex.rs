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
// evicts, so 600 patterns that cycle compile 600 times; 5,000 distinct ones are past the 1,024
// small patterns a call may hold, `#ERROR!` (DEVIATION, decision 0014).
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

// The replacement keeps the regex crate's syntax, as on main: `$$` is a dollar, `${name}` and
// `$name` a named group, `$1a` the (absent) group named `1a`, and a lone `$` itself. UNVERIFIED
// against Excel's PCRE2 syntax (T-1817 probes it).
#[test]
fn regexreplace_replacement_syntax() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=REGEXREPLACE(\"ab\",\"(?P<x>a)(b)\",\"[${x}|$2|$$|$1a|$]\")",
    );
    model._set("A2", "=REGEXREPLACE(\"ab\",\"(?P<x>a)\",\"$x$x\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"[a|b|$||$]");
    assert_eq!(model._get_text("A2"), *"aab");
}
