---
id: T-1973
title: "PCRE2 pattern parser for the REGEX functions, regular patterns on the automata"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
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
- regexreplace-limit-and-syntax A23: Unicode `[[:alpha:]]`;
- probes/regex-optional-args A24-A26: simple case folding.
Backtracking-only constructs parse but stay `#VALUE!` until T-1900: backreferences, lookaround, atomic and possessive
groups, `\K`, conditionals, `\G`. Use the nest limit from T-1972 if it has landed, else 250.

## Acceptance criteria
- [ ] The parser is iterative (no recursion on pattern size), caps nesting, and fails closed (`#VALUE!`) on every
  construct it does not know.
- [ ] regex-pcre2-features C14-C19 (those that need no backtracking) and regexreplace-limit-and-syntax A23 pass, with
  no new known failures in eval/baseline/.
- [ ] A differential unit test checks our parser's `Hir` against `regex-syntax`'s for random patterns both dialects
  read alike.
- [ ] `regex-syntax` becomes a direct dependency pinned to the locked version, with a decision record if
  `cargo deny` or the policy requires one.
- [ ] The T-1272 lift benchmarks for REGEX* do not regress beyond noise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The backtracker (T-1900).
- Iteration (T-1899).
