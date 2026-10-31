---
id: T-3547
title: "PERCENTOF with an error cell argument is #NUM! in Excel (engine propagates the error)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
PERCENTOF returns `#NUM!` when an argument is (or sums to) the `#DIV/0!` error cell, as Excel does, instead of propagating `#DIV/0!`.

## Context
Found by the T-3494 arg-pairs batch (excel-graph oracle). Data!G9 is `#DIV/0!` in gen-m5/f17765c5247b826e; every PERCENTOF call with it as `part` or `total` (alone or beside a number or range) is `#NUM!` in Excel and `#DIV/0!` in the engine (`fn_percentof`/`percent_of`, crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs).
- gen-m5/f17765c5247b826e: A16, A64, A100, A130, A172, A232, A286, A340, A352, A358, A364, A370, A376, A382, A388, A394, A400, A406, A412, A454, A478 (21 cells).
- Probe first which error types give `#NUM!` (other error kinds are unevidenced; use the excel-probe skill) and whether the rule is per-argument; record it in docs/excel-semantics/ (numbers.md or arrays.md PERCENTOF section). GROUPBY/PIVOTBY share `percent_of`; check their passing cells do not regress.

## Visible-parity stage
Decision 0083: a wrong error type is visible to every user of the function; the acceptance criteria below are already the visible-parity stage.

## Acceptance criteria
- [ ] The 21 PERCENTOF cells above match Excel, with the rule probed for at least `#N/A` and `#VALUE!` and documented.
- [ ] The cells above leave eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' error handling.
