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
    model._set("A2", "=REGEXTEST(\"text\", \".\", 0, \"extra\")");
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

// return_mode 0 is the whole match even with capture groups; 2 is the groups, in a row
// (probes/regex-optional-args A29-A36).
#[test]
fn test_regexextract_capture_group() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=REGEXEXTRACT(\"2024-01-15\", \"(\\d{4})-\\d{2}-\\d{2}\")",
    );
    model._set(
        "A2",
        "=REGEXEXTRACT(\"2024-01-15\", \"(\\d{4})-(\\d{2})-\\d{2}\", 2)",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2024-01-15");
    assert_eq!(model._get_text("A2"), "2024");
    assert_eq!(model._get_text("B2"), "01");
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
    model._set("A2", "=REGEXEXTRACT(\"text\", \".\", 0, 0, \"extra\")");
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
        (r#""zz","(a)","$2""#, "zz"),
        (r#""zz","(a)","\q""#, "zz"),
        (r#""b","(a)?b","[$1]""#, "[]"),
        (r#""a","a","[\-]""#, "[-]"),
        (r#""ab","a","""#, "b"),
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
    model._set(
        "A2",
        "=REGEXREPLACE(\"text\", \".\", \"x\", 0, 0, \"extra\")",
    );
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

// T-1817: REGEXREPLACE knows its result length before it builds it. Excel caps the result at
// 65,535 UTF-16 units and is `#CALC!` past it (probes/regexreplace-result-cap,
// probes/regexreplace-limit-and-syntax A1, A2); an empty pattern over 32,767 characters with a
// 10-character replacement is refused without building its 360,000 characters.
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
        "=LEN(REGEXREPLACE(REPT(\"ab\",16000),\"(a)(b)\",\"$2$1$1$1$1\"))",
    );
    model._set(
        "A5",
        "=LEN(REGEXREPLACE(REPT(\"ab\",10000),\"(a)(b)\",\"$2$1$1\"))",
    );
    model._set(
        "A6",
        "=LEN(REGEXREPLACE(REPT(\"a\",16384)&REPT(\"c\",16383),\"a\",\"bbb\"))",
    );
    model._set(
        "A7",
        "=LEN(REGEXREPLACE(REPT(\"a\",16385)&REPT(\"c\",16381),\"a\",\"b\"&UNICHAR(128512)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#CALC!");
    assert_eq!(model._get_text("A2"), *"65534");
    assert_eq!(model._get_text("A3"), *"32766");
    assert_eq!(model._get_text("A4"), *"#CALC!");
    assert_eq!(model._get_text("A5"), *"30000");
    assert_eq!(model._get_text("A6"), *"65535");
    // 65,536 units in 49,150 characters: the cap counts units.
    assert_eq!(model._get_text("A7"), *"#CALC!");
}

// T-1829: iterating matches on the PikeVM (a `\b` pattern over non-ASCII text, where the lazy
// DFA quits) is charged the states a scan of the NFA visits, not the rest of the text for every
// match, so a few KB with many matches returns values (T-1272 made these `#ERROR!`); ASCII text
// stays on the lazy DFA.
#[test]
fn pike_iteration_is_charged_by_work_done() {
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
    model._set(
        "A5",
        "=LEN(REGEXREPLACE(REPT(\"café foo bar \",250),\"\\b(\\w+)\\b\",\"<$1>\"))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"300");
    assert_eq!(model._get_text("A2"), *"500");
    assert_eq!(model._get_text("A3"), *"27500");
    assert_eq!(model._get_text("A4"), *"27500");
    assert_eq!(model._get_text("A5"), *"4750");
}

// T-1829: a search on the PikeVM runs over the span the scan measured, which must not change
// its leftmost-first match: a higher-priority alternative that reads past a lower one's match
// still wins, and empty matches still fall between characters.
#[test]
fn pike_search_span_keeps_leftmost_first_matches() {
    let mut model = new_empty_model();
    model._set("A1", "=REGEXEXTRACT(\"é aéééz a\",\"\\ba.*z|\\ba\")");
    model._set(
        "A2",
        "=TEXTJOIN(\"|\",FALSE,REGEXEXTRACT(\"é aéz aé\",\"\\ba.*z|\\ba\",1))",
    );
    model._set("A3", "=REGEXREPLACE(\"éa é\",\"\\b\",\"-\")");
    model._set("A4", "=REGEXREPLACE(\"éé\",\"(?:\\b|)\",\"-\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"aéééz");
    assert_eq!(model._get_text("A2"), *"aéz|a");
    assert_eq!(model._get_text("A3"), *"-éa- -é-");
    assert_eq!(model._get_text("A4"), *"-é-é-");
}

// T-1829: a truly quadratic iteration (each search reads to the end of the text, where the
// lazy DFA quits on the `é`) is still charged every byte it reads, so it stops (DEVIATION,
// decision 0014).
#[test]
fn quadratic_pike_iteration_is_still_bounded() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=LEN(REGEXREPLACE(REPT(\"a\",20000)&\"é\",\"a.*z|a|\\bq\",\"b\"))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
