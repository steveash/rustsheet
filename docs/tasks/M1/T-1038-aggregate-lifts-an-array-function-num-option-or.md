---
id: T-1038
title: "AGGREGATE lifts an array function_num, option or k"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
AGGREGATE with an array function_num, option or k gives Excel's values.

## Context
Found by T-0993. Excel gives 23 for `AGGREGATE({9,4},0,E1:E6)`, `#DIV/0!` for `AGGREGATE(9,{0,4},D1:D11)` and 10 for `AGGREGATE(14,6,D1:D11,{1,2})` in the checked cell; the engine gives `#N/IMPL!` (probes/aggregate-args Misc!A36:A38, known failures). Probe whether these spill (check the neighbouring cell) before choosing lift versus first element. Code: `fn_aggregate` (crates/rustsheet-engine/src/functions/aggregate.rs), docs/excel-semantics/numbers.md "AGGREGATE". Static analysis gives AGGREGATE `args_signature_npv` (options and k marked Vector), which lifting must revisit.

## Acceptance criteria
- [ ] probes/aggregate-args Misc!A36:A38 pass, and the spill shape (if any) is probed and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
