#![allow(clippy::unwrap_used)]

use regex_syntax::hir::Hir;

use super::{
    parse::{parse, Invalid, NEST_LIMIT},
    translate, FinalLf, Form, ProgramFailure,
};

/// `pattern` translated for the automata; panics on a pattern only the backtracker runs.
fn hir(pattern: &str, caseless: bool, final_lf: FinalLf) -> Hir {
    match translate(pattern, caseless, final_lf, usize::MAX)
        .unwrap()
        .form
    {
        Form::Automata(hir) => hir,
        Form::Backtrack(_) => panic!("{pattern} is not regular"),
    }
}

/// Whether `pattern` matches `text`, translated as REGEXTEST translates it.
fn matches(pattern: &str, text: &str, caseless: bool) -> bool {
    regex_automata::meta::Regex::builder()
        .build_from_hir(&hir(pattern, caseless, super::final_lf_for(text)))
        .unwrap()
        .is_match(text)
}

fn invalid(pattern: &str) -> bool {
    translate(pattern, false, FinalLf::EndLf, usize::MAX).is_err()
}

#[test]
fn pcre2_only_syntax_is_read() {
    // probes/regex-pcre2-features C14-C19: comments, \Q..\E, (?x), (?-i).
    assert!(matches("a(?#note)b", "ab", false));
    assert!(matches(r"\Q.\E", ".", false));
    assert!(!matches(r"\Q.\E", "a", false));
    assert!(matches(r"\Q[[:alpha:]]\E", "[[:alpha:]]", false));
    assert!(matches("(?x) a b # comment", "ab", false));
    assert!(matches("(?i)a(?-i)b", "Ab", false));
    assert!(!matches("(?i)a(?-i)b", "AB", false));
    assert!(matches("(?i:a)b", "Ab", false));
    // (*UCP), \h (probes/regex-ascii-classes A17, A41).
    assert!(matches(r"(*UCP)\w", "é", false));
    assert!(matches(r"\h", "\t", false));
    assert!(!matches(r"\h", "\n", false));
}

#[test]
fn case_folding_is_simple() {
    // probes/regex-optional-args A24-A26.
    assert!(matches("s", "\u{17F}", true));
    assert!(matches("k", "\u{212A}", true));
    assert!(matches("(?i)K", "k", false));
    assert!(!matches("K", "k", false));
}

#[test]
fn dollar_matches_before_a_final_line_feed() {
    // probes/regex-ascii-classes A34, A32, A35.
    assert!(matches("a$", "a\n", false));
    assert!(matches(r"a\Z", "a\n", false));
    assert!(!matches(r"a\z", "a\n", false));
    assert!(!matches("a$", "a\nb", false));
    assert!(matches("a$", "a", false));
    // A text with another \n keeps `\z` until T-1900's memoized mode: PCRE2 matches here.
    assert!(!matches("a$", "b\na\n", false));
}

#[test]
fn no_posix_class_folds_while_the_other_items_do() {
    // probes/regex-posix-classes-b A8-A11, -c A1-A26 (excel-graph, T-1989).
    for (pattern, text, expected) in [
        ("(?i)[[:upper:]]", "a", false),
        ("(?i)[[[:upper:]]]", "a", false),
        ("(?i)[[[:upper:]]]", "A", true),
        ("(?i)[^[:upper:]]", "a", true),
        ("(?i)[[:^upper:]]", "a", true),
        ("(?i)[[:^lower:]]", "A", true),
        ("(?i)[^[:lower:]]", "a", false),
        ("(?i)[^[:upper:]x]", "X", false),
        ("(?i)[^[:upper:]x]", "x", false),
        ("(?i)[^[:upper:]x]", "B", false),
        ("(?i)[^[:upper:]x]", "b", true),
        ("(?i)[[:^upper:]x]", "X", true),
        ("(?i)[[:lower:]a-c]", "B", true),
        ("(?i)[[:lower:]a-c]", "D", false),
        ("(?i)[[:upper:]&&[:alpha:]]", "a", false),
        ("(?i:[[:upper:]])", "a", false),
        ("(?i)(?-i)(?i)[^[:^upper:]]", "a", false),
        ("(?i)[[:ascii:]]", "\u{212A}", false),
        ("(?i)[[:ascii:]]", "\u{17F}", false),
        ("(?-i)[[:upper:]x]|(?i)[[:upper:]x]", "X", true),
        ("[[:upper:]x]", "X", true),
        ("[[a]]", "a", true),
        ("[[a]]", "[", false),
        ("[[:alpha:]&&[:lower:]]", "A", false),
        ("[[:alpha:]]", "é", true),
    ] {
        assert_eq!(
            matches(pattern, text, false),
            expected,
            "{pattern} on {text}"
        );
    }
    assert!(!matches("[^[:upper:]x]", "X", true));
    assert!(matches("[^[:upper:]x]", "b", true));
    assert!(matches("[[:lower:]x]", "X", true));
    assert!(!matches("(?-i)[[:lower:]x]", "X", true));
}

#[test]
fn posix_syntax_pcre2_rejects_is_invalid() {
    for pattern in [
        "[:alpha:]",
        "[.a.]",
        "[..]",
        "[[:foo:]]",
        "[[:ALPHA:]]",
        "[[.a.]]",
        "[[=a=]]",
        "[[:alpha:]-z]",
        "[a[:upper:]-z]",
        "[a-[:alpha:]]",
        "x(?:[[.a.]])+",
    ] {
        assert!(invalid(pattern), "{pattern}");
    }
    for pattern in [
        r"[\[:alpha:]]",
        "[[:alpha:]-]",
        "[-[:alpha:]]",
        "[:a]",
        "[a:]",
        "[[:alpha:]]",
    ] {
        assert!(!invalid(pattern), "{pattern}");
    }
}

#[test]
fn backtracking_constructs_go_to_the_backtracker() {
    // Each parses and is not regular. T-1900 runs the first four; the rest wait for T-2354
    // (lookaround, `\K`, conditionals) and T-2287 (`\R`, `\X`).
    let program = |pattern: &str| match translate(pattern, false, FinalLf::EndLf, usize::MAX)
        .unwrap()
        .form
    {
        Form::Backtrack(backtrack) => backtrack.compile(usize::MAX),
        Form::Automata(_) => panic!("{pattern} is regular"),
    };
    for pattern in ["a++", "(?>a)", r"(a)\1", r"\Ga"] {
        assert!(program(pattern).is_ok(), "{pattern}");
    }
    for pattern in ["(?=a)", "(?<!a)b", r"a\Kb", "(a)(?(1)b|c)", r"\R", r"\X"] {
        assert!(
            matches!(program(pattern), Err(ProgramFailure::Unsupported)),
            "{pattern}"
        );
    }
}

#[test]
fn unknown_constructs_fail_closed() {
    for pattern in [
        "(", ")", "a{2,1}", "*", "(?P>x)", "(*FAIL)", r"\v", r"\N", "a{,3}", "(?|a)", "a{1, 2}",
        "a{ 2}", "[a--b]", "[a~~b]", "[a||b]",
    ] {
        assert!(invalid(pattern), "{pattern}");
    }
    for pattern in ["x{ }", "[a|b]", "[a-]", "[a~b]"] {
        assert!(!invalid(pattern), "{pattern}");
    }
    // A brace that starts no quantifier is a literal in PCRE2.
    assert!(matches("a{x", "a{x", false));
}

#[test]
fn nesting_and_repeat_counts_are_capped() {
    let deep = |n: usize| format!("{}a{}", "(".repeat(n), ")".repeat(n));
    assert!(parse(&deep(NEST_LIMIT as usize), false, usize::MAX).is_ok());
    assert_eq!(
        parse(&deep(NEST_LIMIT as usize + 1), false, usize::MAX).err(),
        Some(Invalid)
    );
    let stacked = format!("a{}", "{1}".repeat(32_767));
    assert_eq!(parse(&stacked, false, usize::MAX).err(), Some(Invalid));
    assert!(parse("a{65535}", false, usize::MAX).is_ok());
    assert_eq!(parse("a{65536}", false, usize::MAX).err(), Some(Invalid));
    // Parses; the NFA's size limit refuses it, with no overflow on the way.
    let huge = hir("(?:a{65535}){65535}", false, FinalLf::EndLf);
    assert!(regex_automata::nfa::thompson::NFA::compiler()
        .configure(regex_automata::nfa::thompson::NFA::config().nfa_size_limit(Some(10 << 20)))
        .build_from_hir(&huge)
        .is_err());
    let long = "a".repeat(32_767);
    assert!(parse(&long, false, usize::MAX).is_ok());
}

#[test]
fn folding_stops_at_its_budget() {
    let ranges = r"[\x{0}-\x{10FFFF}]".repeat(1_000);
    let start = std::time::Instant::now();
    let translated = translate(&ranges, true, FinalLf::EndLf, 1 << 27).unwrap();
    assert_eq!(translated.fold_cost, usize::MAX);
    assert!(start.elapsed().as_secs() < 5);
    assert_eq!(
        translate("[a-c]", true, FinalLf::EndLf, 100)
            .unwrap()
            .fold_cost,
        24
    );
}

#[test]
fn our_hir_agrees_with_regex_syntax_where_the_dialects_agree() {
    // Patterns both dialects read alike (no \Q, possessive or POSIX class): the same Hir.
    let atoms = [
        "a", "b", ".", r"\d", r"\w", "[a-c]", "[^b]", r"\b", "^", "(a)", "(?:ab)", "(?i)a",
    ];
    let quantifiers = ["", "*", "+?", "?", "{2}", "{1,3}"];
    let mut seed: u32 = 12_345;
    let mut next = |n: usize| {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (seed >> 16) as usize % n
    };
    for _ in 0..2_000 {
        let mut pattern = String::new();
        for _ in 0..=next(4) {
            if !pattern.is_empty() && next(5) == 0 {
                pattern.push('|');
            }
            let atom = atoms[next(atoms.len())];
            pattern.push_str(atom);
            // An assertion takes no quantifier in PCRE2.
            if !matches!(atom, "^" | r"\b") {
                pattern.push_str(quantifiers[next(quantifiers.len())]);
            }
        }
        let ours = hir(&pattern, false, FinalLf::EndLf);
        let theirs = regex_syntax::Parser::new().parse(&pattern).expect(&pattern);
        assert_eq!(ours, theirs, "{pattern}");
    }
}

#[test]
fn nested_lookbehind_alternations_are_measured_in_linear_time() {
    let mut inner = String::from("a|b");
    for _ in 0..200 {
        inner = format!("(?:{inner})|c");
    }
    let start = std::time::Instant::now();
    assert!(parse(&format!("(?<={inner})x"), false, usize::MAX).is_ok());
    assert!(start.elapsed().as_secs() < 2);
}
