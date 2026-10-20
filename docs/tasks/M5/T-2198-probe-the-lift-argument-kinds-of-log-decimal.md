---
id: T-2198
title: "Probe the lift argument kinds of LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE (T-2159, T-2160)"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for each argument of LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE: does it lift over an
array constant, and does it lift over or refuse a multi-cell range given directly? T-2159 and T-2160 then pick `Lift` or
`LiftArrays` per argument from the case and do not have to probe.

## Context
- The M5 exit needs gen-m5 at 99.9% (decision 0085 item 5): at most 22 known failures; 190 are left today. T-2159
  (7 gen-m5 cells) and T-2160 (5 cells) are functions-math and functions-text tasks, and both begin with "probe each
  argument's kind (`excel-probe`) before choosing". Those lanes are full; this task does the probe in the research lane.
- What is known: docs/excel-semantics/arrays.md "LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE return
  arrays over array arguments (T-2159, T-2160)". It says these functions lift over an array or an `INDEX(..,0,..)` result.
  Its "Unprobed" line asks which arguments lift and whether a direct multi-cell range lifts or is refused. Analysis ToolPak
  functions refuse a direct range (numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes"; MROUND in arrays.md
  "CEILING/FLOOR family and MROUND lift over arrays").
- Driver kinds (SPEC-0011): `Lift` lifts over arrays and ranges, and `LiftArrays` lifts over arrays only, so a direct
  multi-cell range is one `#VALUE!`. They are listed per function in eval/catalog/lift-kinds.json
  (`rustsheet-eval lift-kinds`). Follow the case layout of probes/eng-later-argument-lift (T-1383): `ROWS`/`COLUMNS` of
  the result tells a lift from a refusal, and an `INDEX(..,2)` cell gives an element value next to its scalar control.
- Tag the case `M5`: every subject function is M5 in eval/catalog/scope.json (decision 0051).

## Acceptance criteria
- [ ] New case `eval/suites/probes/cases/t2198-m5-lift-argument-kinds.case.json` (`rustsheet-case/v1`, one `M5` tag). For
      every argument position of each of the 8 functions (LOG 2, DECIMAL 2, BASE 3, ATAN2 2, ARABIC 1, DOLLAR 2, FIXED 3,
      NUMBERVALUE 3), with valid scalars in the other arguments, it has:
      (a) `ROWS(f(.., <3x1 range of valid values>, ..))` on rows off the range, so no implicit intersection applies;
      (b) `ROWS(f(.., {v1;v2;v3}, ..))`;
      (c) `INDEX(f(.., {v1;v2;v3}, ..),2)` beside the scalar `f(.., v2, ..)`.
      Also one cell per function with an array in two arguments of different shapes (`{1;2}` and `{1,2,3}`), which shows
      the broadcast shape and its `#N/A` fill.
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`), with no
      pending cell.
- [ ] arrays.md "LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE return arrays over array arguments"
      replaces its "Unprobed" line with a table of the kind each argument shows (lifts over a range, or arrays only), and
      cites the case.
- [ ] The case's failing cells are new-coverage F lines in eval/baseline/probes.txt (`make eval-baseline`), with no
      regression in any suite. T-2159's and T-2160's Contexts name the case as their evidence and as cells they must pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-2159, T-2160).
- Legacy (pre-dynamic-array) formulas of these functions.
