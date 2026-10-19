---
id: T-1973
title: "PCRE2 pattern parser for the REGEX functions, regular patterns on the automata"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1898, T-1830]
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
- probes/regex-optional-args A24-A26: simple case folding.
Backtracking-only constructs parse but stay `#VALUE!` until T-1900: backreferences, lookaround, atomic and possessive
groups, `\K`, conditionals, `\G`. Use the nest limit from T-1972 if it has landed, else 250.

## Acceptance criteria
- [ ] The parser is iterative (no recursion on pattern size), caps nesting, and fails closed (`#VALUE!`) on every
  construct it does not know.
- [ ] regex-pcre2-features C14-C19 pass (`(?#..)`, `\Q..\E`, `(?x)`, `(?-i)`; none needs backtracking), as do
  probes/regex-optional-args A24-A26 (case folding), with no new known failures in eval/baseline/. POSIX classes keep
  their current meaning (T-1903 changes it).
- [ ] Every AST pass (parse, `is_regular`, `to_hir`, `Drop`) is iterative or depth-bounded. Stacked quantifiers count
  toward the nest cap, and repeat counts are capped at 65,535 with checked arithmetic. Unit tests cover 250 nested
  groups, 32,767 stacked quantifiers and `(?:a{65535}){65535}` (`TooBig`, no overflow).
- [ ] `$` and `\Z` keep today's meaning (`\z`) until T-1972 shows PCRE2's final-`\n` rule (SPEC-0013 section 2).
- [ ] A unit test pins `a++` as `#VALUE!` until T-1900.
- [ ] A differential unit test checks our parser's `Hir` against `regex-syntax`'s for random patterns both dialects
  read alike.
- [ ] `regex-syntax = "0.8"` is a direct dependency with one locked copy (`cargo tree -d`) and a short decision record.
- [ ] T-1830's REGEX* benchmarks, plus `^\d{5}$` over a column and many distinct small patterns in one call, have
  before and after numbers in the PR. A slowdown over 5% is justified there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The backtracker (T-1900).
- Iteration (T-1899).
