---
id: T-2106
title: "TEXT of DBL_MAX with 0.00E+00 gives 1.80E+308 (probes/dbl-max-text-b)"
milestone: M9
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(2^1023*(2-2^-52),"0.00E+00")` matches Excel.

## Context
probes/dbl-max-text-b A1 (known failure in eval/baseline/probes.txt) expects `1.80E+308` (excel-graph). Found while closing T-2101; see engineering.md (IMEXP section). Ours prints `1.80E+30308` (exponent formatting; may affect any 3-digit exponent with an `E+00` format). Also add the DBL_MAX number-to-text quirks to the TEXT/numbers semantics docs.

## Acceptance criteria
- [ ] probes/dbl-max-text-b passes and leaves the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
