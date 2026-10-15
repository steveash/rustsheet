---
id: T-1585
title: "Compare, EXACT, MATCH and sort lone surrogates by code unit (not as U+FFFD)"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1580]
---

## Goal
`=`, `<`, EXACT and MATCH treat a lone surrogate as its own code unit, never as U+FFFD.

## Context
probes/text-lone-surrogates A10-A28 and docs/excel-semantics/text.md "Lone surrogates through CODE, comparisons...". Depends on SPEC-0010 `Text` (T-1575..T-1580): the lossy view makes EXACT(H,L) TRUE today. Note the ordering quirk: H<L, but H<U+FFFD and L<U+FFFD are FALSE.

## Acceptance criteria
- [ ] probes/text-lone-surrogates A10-A28 pass (EXACT, =, <, MATCH).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
