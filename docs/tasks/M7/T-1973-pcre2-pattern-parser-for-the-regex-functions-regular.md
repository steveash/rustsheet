---
id: T-1973
title: "PCRE2 pattern parser for the REGEX functions, regular patterns on the automata"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-1898]
---

## Goal
Every REGEX* pattern is parsed by one PCRE2 parser (SPEC-0013 section 1). Regular patterns run on the existing
automata through a translated `Hir` (section 2), so the `regex` crate's syntax is no longer our dialect.

## Context
SPEC-0013 (docs/specs/SPEC-0013-pcre2-regex-matcher.md), rollout step 2. Code: new module
crates/rustsheet-engine/src/functions/text/pcre/ (`parse.rs`, `ast.rs`, `hir.rs`, `mod.rs` with `Pattern`);
functions/text/regex_functions.rs builds through `Pattern` instead of `regex_engine::Engine::new`, and `Engine`
takes a `Hir` (`Compiler::build_from_hir`). Evidence:
- probes/regex-pcre2-features C14-C19: `(?#..)`, `\Q..\E`, `(?x)`, `(?-i)`;
- probes/regex-optional-args A24-A26: simple case folding;
- probes/regex-ascii-classes A34 (`$` before a final LF), A17 (`(*UCP)`), A41 (`\h`), and A27, A42 (`\X`, `\R`
  accepted; text.md "REGEX* character classes are Unicode; `$` and `.` around line breaks (T-1829)");
- the bracket rules of T-1903, T-1985, T-1989 and T-2010 (text.md "REGEX* POSIX classes are Unicode (T-1903)",
  SPEC-0013 section 1): nested brackets, `&&`, POSIX classes unfolded under `(?i)` while the other items fold, and
  the POSIX syntax errors.
Backtracking-only constructs parse but stay `#VALUE!` until T-1900: backreferences, lookaround, atomic and possessive
groups, `\K`, conditionals, `\G`, and `\R` and `\X`. Use the nest limit from T-1972 if it has landed, else 250.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 6 cells, 0 numeric past half a unit in the 15th digit and 6 with a wrong type, error or text, blamed on REGEXTEST 5, REGEXEXTRACT 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/regex-ascii-classes Sheet1!A17 | `=IFERROR(REGEXTEST("é","(*UCP)\w"),"E"&ERROR.TYPE(REGEXTEST("é","(*UCP)\w")))` | TRUE | "E3" |
| probes/regex-ascii-classes Sheet1!A34 | `=IFERROR(REGEXTEST("a"&CHAR(10),"a$"),"E"&ERROR.TYPE(REGEXTEST("a"&CHAR(10),"a$")))` | TRUE | FALSE |
| probes/regex-ascii-classes Sheet1!A41 | `=IFERROR(REGEXTEST(CHAR(9),"\h"),"E"&ERROR.TYPE(REGEXTEST(CHAR(9),"\h")))` | TRUE | "E3" |
| probes/regex-pcre2-features Sheet1!C15 | `=REGEXTEST("ab","a(?#note)b")` | TRUE | #VALUE! |
| probes/regex-pcre2-features Sheet1!C16 | `=REGEXEXTRACT("a.b","\Q.\E")` | "." | #VALUE! |
| probes/regex-posix-classes Sheet1!Z7 | `=IFERROR(REGEXTEST("[[:alpha:]]","\Q[[:alpha:]]\E"),"err "&ERROR.TYPE(REGEXTEST("[[:alpha:]]","\Q[[:alpha:]]...` | TRUE | "err 3" |

## Acceptance criteria
- [ ] Visible parity: the 6 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 6 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The parser is iterative (no recursion on pattern size), caps nesting, and fails closed (`#VALUE!`) on every
  construct it does not know.
- [ ] regex-pcre2-features C14-C19 pass (`(?#..)`, `\Q..\E`, `(?x)`, `(?-i)`; none needs backtracking), as do
  probes/regex-optional-args A24-A26 (case folding), with no new known failures in eval/baseline/. POSIX classes keep
  the Unicode meaning T-1903 gave them (`regex_posix.rs`, probes/regex-posix-classes and -b pass), and the parser
  replaces that pre-compile rewrite. The bracket probes keep passing: probes/regex-posix-classes, -b and -c
  (nesting, `&&`, the `(?i)` rule; T-1903, T-1989) and the syntax-error cells (regex-posix-classes Z1-Z3, Z8-Z10,
  regex-posix-classes-b A4, A5, T-1985; regex-posix-syntax-edges A1-A14, T-2010).
- [ ] Every AST pass (parse, `is_regular`, `to_hir`, `Drop`) is iterative or depth-bounded. Stacked quantifiers count
  toward the nest cap, and repeat counts are capped at 65,535 with checked arithmetic. Unit tests cover 250 nested
  groups, 32,767 stacked quantifiers and `(?:a{65535}){65535}` (`TooBig`, no overflow).
- [ ] `$` and `\Z` match before a final `\n` (SPEC-0013 section 2): `Look::End` on a text that does not end in `\n`,
  `Look::EndLF` on one whose only `\n` is the final one, so regex-ascii-classes A34 passes. A text that ends in `\n`
  and holds another one keeps `\z` until T-1900's memoized mode, pinned by a unit test.
- [ ] `\h` and a leading `(*UCP)` are accepted (regex-ascii-classes A41, A17 pass). `\R` and `\X` (A42, A27) parse
  and stay `#VALUE!`; T-2259 (functions-text, depends on T-1900) implements them.
- [ ] A unit test pins `a++` as `#VALUE!` until T-1900.
- [ ] A differential unit test checks our parser's `Hir` against `regex-syntax`'s for random patterns both dialects
  read alike.
- [ ] `regex-syntax = "0.8"` is a direct dependency with one locked copy (`cargo tree -d`) and a short decision record.
- [ ] Before and after timings of REGEX* over a column, `^\d{5}$` over a column and many distinct small patterns
  in one call are in the PR, measured ad hoc (T-1830's benchmark suite is not required; dropped as a dependency
  2026-10-21 so this campaign task does not wait on the perf lane). A slowdown over 50% is justified there
  (decision 0081).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The backtracker (T-1900).
- Iteration (T-1899).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` or FALSE where Excel accepts `(?#..)`, `\Q..\E`, `(*UCP)`, `\h` and `$` before a final LF), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. `\X` and `\R` (A27, A42) are T-2259.
