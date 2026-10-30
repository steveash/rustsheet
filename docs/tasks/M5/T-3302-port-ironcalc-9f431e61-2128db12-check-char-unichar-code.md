---
id: T-3302
title: "Port ironcalc 9f431e61..2128db12: check CHAR, UNICHAR, CODE, UNICODE, CONCAT and TEXTJOIN over arrays and ranges against Excel evidence"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CHAR, UNICHAR, CODE, UNICODE, CONCAT and TEXTJOIN over ranges and arrays (for example
`CONCAT(CHAR(A1:A5))`) give Excel's results, backed by oracle evidence, with any gap fixed.

## Context
Upstream IronCalc 9f431e61 (CHAR), 69628c5a (UNICHAR), 42de521a (CODE, TEXTJOIN), ac94983f
(UNICODE) and 2128db12 (CONCAT) made these functions accept ranges and arrays; upstream had
returned `#N/IMPL!` for them. Our engine already lifts CHAR/UNICHAR/CODE/UNICODE
(`functions/lift/kinds/text.rs`) and joins arrays in CONCAT/TEXTJOIN, but
`functions/text/concatenate.rs` still has two `"Arrays not supported yet"` `#N/IMPL!` arms (an
array met as a range cell's value or a value item). Upstream's unit tests are IronCalc-authored,
not evidence: find existing probes first (`rg` in eval/suites/probes), then use the
`excel-probe` skill for the shapes upstream tests (nested CHAR in CONCAT/TEXTJOIN, a spilled
range of CODE, an error element in the array) and port only where Excel agrees. Triaged by the
2026-10-30 18:47 UTC steward run.

## Acceptance criteria
- [ ] A probe case (or cited existing cases) covers each function over a range and an inline or computed array, oracle-filled.
- [ ] Every probed cell matches Excel or is a recorded known failure with an owner; any `#N/IMPL!` arm reached by a probed shape is gone.
- [ ] Code taken from upstream carries `Ported-From: ironcalc@<sha>`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream's ENCODEURL and defined-name commits (T-3300, T-3301).
