---
id: T-1900
title: "REGEX functions: backreferences, lookaround, atomic and possessive groups, K, conditionals"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1973]
---

## Goal
The REGEX* functions accept the PCRE2 pattern features Excel accepts and match as Excel does.

## Context
Found by T-1688 (probes/regex-pcre2-features C1-C19, B27-B29; t1626-regextest C29): backreferences `\1`, `\g1`, `\k<n>` (case-insensitive too), lookahead, fixed-length lookbehind (`(?<=a+)` is `#VALUE!`), possessive `a++` and atomic `(?>...)` (no match: `#N/A`), `\K`, conditionals, `(?#...)` comments, `\Q...\E`, `(?x)` and `(?-i)`. Design: SPEC-0013 section 3 (rollout step 3). Compile T-1973's `Ast` to `pcre::Program` (flat instructions, counted
repeats) and run it on `pcre::Backtracker`. The backtracker has one explicit stack, no recursion, and barrier frames
for lookaround and atomic groups. Possessive quantifiers compile as atomic groups. It also needs PCRE2's empty-loop
rule. Every instruction and frame is charged to the call's `Spent` budget (placeholder costs; T-1974 fits them), and
the stack is capped in bytes. `Pattern` routes non-regular patterns here. Add a differential test that runs random
regular patterns on both engines and compares the match and capture spans.

## Acceptance criteria
- [ ] t1626-regextest C29 and regex-pcre2-features C1-C19 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iteration after empty matches (T-1899).
