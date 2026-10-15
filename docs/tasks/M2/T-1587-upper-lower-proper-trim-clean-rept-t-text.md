---
id: T-1587
title: "UPPER, LOWER, PROPER, TRIM, CLEAN, REPT, T, TEXT keep lone surrogates; update SPEC-0010 with T-1582 findings"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1580]
---

## Goal
UPPER, LOWER, PROPER, TRIM, CLEAN, REPT, T, TEXT and `&` keep lone surrogates exactly; SPEC-0010 records that `&` re-pairs high+low and that a lone high is cached as `_xD83D_`.

## Context
probes/text-lone-surrogates A5-A9, A29-A40, A62-A80, B1-B6. Replace the "provisional" wording in SPEC-0010 with the evidence.

## Acceptance criteria
- [ ] probes/text-lone-surrogates A5-A9, A29-A40, A62-A80, B1-B6 pass; SPEC-0010 'Unprobed' updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
