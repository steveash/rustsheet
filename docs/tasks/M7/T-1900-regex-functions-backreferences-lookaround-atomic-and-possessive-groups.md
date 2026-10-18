---
id: T-1900
title: "REGEX functions: backreferences, lookaround, atomic and possessive groups, K, conditionals"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1898]
---

## Goal
The REGEX* functions accept the PCRE2 pattern features Excel accepts and match as Excel does.

## Context
Found by T-1688 (probes/regex-pcre2-features C1-C19, B27-B29; t1626-regextest C29): backreferences `\1`, `\g1`, `\k<n>` (case-insensitive too), lookahead, fixed-length lookbehind (`(?<=a+)` is `#VALUE!`), possessive `a++` and atomic `(?>...)` (no match: `#N/A`), `\K`, conditionals, `(?#...)` comments, `\Q...\E`, `(?x)` and `(?-i)`. Follows the matcher chosen by T-1898; the spec may split this further.

## Acceptance criteria
- [ ] t1626-regextest C29 and regex-pcre2-features C1-C19 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iteration after empty matches (T-1899).
