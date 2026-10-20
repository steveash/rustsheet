--- Changing which patterns match.
id: T-2138
title: "Charge regex-syntax case folding of non-POSIX (?i) brackets to the compile budget"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
--- Changing which patterns match.

## Goal
A REGEX* call whose `(?i)` pattern has no POSIX class but huge ranges (`(?i)[\x{0}-\x{10FFFF}]`×1000) is bounded by the compile budget, instead of spending seconds in case folding inside `Engine::new`.

## Context
Follow-up of T-2098, which charges folding only inside the POSIX rewrite (`folding_cost`, regex_posix.rs). Patterns without POSIX syntax skip `unicode_posix_classes` and are folded by regex-syntax in `Engine::new` (`PatternCache::compile`, regex_functions.rs), charged only by size limit. Past the budget the call is refused (DEVIATION, decision 0014); record it in docs/excel-semantics/text.md.

## Acceptance criteria
- [ ] A unit test shows many full-range `(?i)` brackets without POSIX classes hit the budget well under 1 s (release).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which patterns match.
