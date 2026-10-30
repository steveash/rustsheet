---
id: T-3178
title: "Probe typed input and criteria for sign-space text, one-digit-year dates and sign/currency/percent mixed fractions"
milestone: M9
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish and match what typed cell input and COUNTIF-style criteria do with `- 5`, `+ 5`, `1-1-1`, `1/1/1` and the extrapolated mixed-fraction forms.

## Context
T-2646 changed the shared grammar (`parse_number`, `parse_separated`, `mixed_fraction`), probed only through VALUE and coercion. UNVERIFIED in docs/excel-semantics/text.md: typed input (value and number format), criteria `"- 5"`, and `€3 1/2`, `-$3 1/2`, `-3 1/2%`, `$3 1/2%`, `+ 3 1/2`, `="+ 5"+0`.

## Acceptance criteria
- [ ] An oracle-filled probe covers each form above; our behavior matches it or a fix lands.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
