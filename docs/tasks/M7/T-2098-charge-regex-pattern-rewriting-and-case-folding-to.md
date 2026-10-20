---
id: T-2098
title: "Charge REGEX* pattern rewriting and case folding to the compile budget"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A REGEX* call whose pattern is slow to rewrite or case-fold is bounded by the same work budget as its compiles, so many distinct such patterns in one lifted call cannot take seconds each.

## Context
T-1989 performance review: `COMPILE_BUDGET` (functions/text/regex_functions.rs, `compile`) charges only the
`Engine::new` attempts, by size limit. Neither the POSIX rewrite (`unicode_posix_classes`, regex_posix.rs) nor
regex-syntax's case folding inside `Engine::new` is charged by its cost. Measured in a release build:
`(?i)` + `[[:upper:]\x{0}-\x{10FFFF}]`×1000 (27 KB) takes about 6 s in the rewrite, and about 23 s inside
`Engine::new` before T-1989. Charge an estimate first, for example the total span or count of the ranges to fold, or
the printed size the rewrite spends from `MAX_UNFOLDED_LEN`. Past the budget the call is `too_much_regex_work`
(a DEVIATION, decision 0014).

## Acceptance criteria
- [ ] A unit test shows a pattern of many full-range folded brackets returns the budget error quickly (well under 1 s, release).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which patterns match (T-1989, T-2092).
