---
id: T-1587
title: "UPPER, LOWER, PROPER, TRIM, CLEAN, REPT, T, TEXT keep lone surrogates; update SPEC-0010 with T-1582 findings"
milestone: M2
lane: functions-text
status: done
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
- [x] probes/text-lone-surrogates A5-A9, A29-A40, A62-A80, B1-B6 pass; SPEC-0010 'Unprobed' updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
UPPER, LOWER, PROPER, TRIM, CLEAN, T and `&` already kept lone surrogates (T-1579..T-1586), and probes/text-lone-surrogates passed 87/87 because it only reads lengths. The remaining gaps were REPT (repeated the lossy view) and TEXT (formatted it): added `Text::repeat` and `format_text_value_exact` (one-character private-use stand-in, exact text swapped in, 255-character limit re-checked on the real length), with a unit test. SPEC-0010 "Unprobed" and text.md now record the T-1582 evidence. Eval delta: none (case already fully passing); `make check` green.
