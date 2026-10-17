---
id: T-1569
title: "Probe ATANH(1) and ATANH(-1) (infinite results) against Excel"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ATANH(1) and ATANH(-1) give the same value or error as Excel, probed with the oracle.

## Context
Split from T-1202: `f64::atanh` returns an infinity there, not NaN, so the NaN-to-`#NUM!` mapping in `single_number_fn!` does not catch it. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values; the engine matches (or a fix lands).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/atanh-unit-argument (excel-graph): ATANH(+-1) is `#NUM!` at once (IFERROR catches it, ISERR TRUE). The engine
gave an infinity that only became `#NUM!` at the cell, so IFERROR/ISERR missed it; `fn_atanh` now returns `#NUM!` at +-1.
Rule in docs/excel-semantics/numbers.md. Eval delta: +12 cells pass in probes (2 were failing: A6, A7).
