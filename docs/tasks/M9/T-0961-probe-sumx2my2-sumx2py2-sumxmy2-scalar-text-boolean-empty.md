---
id: T-0961
title: "Probe SUMX2MY2/SUMX2PY2/SUMXMY2 scalar text, boolean, empty args and error-vs-#N/A precedence"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior for SUMX2MY2/SUMX2PY2/SUMXMY2 with scalar text, numeric text, boolean and empty
arguments, and error-versus-`#N/A` precedence, is established by oracle probes and implemented.

## Context
T-0944 treats any non-number scalar as one skipped pair (so `SUMXMY2("a","b")` is `#DIV/0!`) and
returns the first error found before the element-count `#N/A` check. Neither choice has Excel
evidence (reviewer finding). Code: `sumx_values` / `sumx_fold` in
`crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs`. Use the `excel-probe` skill;
cases: `SUMXMY2("2",4)`, `SUMXMY2(TRUE,1)`, `SUMXMY2(,1)`, `SUMXMY2(range_with_error,longer_range)`.

## Acceptance criteria
- [ ] Probe cases in eval/suites/probes with oracle values; coercion.md entry updated to VERIFIED.
- [ ] Implementation matches the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other `fn_get_two_matrices` callers.
