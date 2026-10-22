---
id: T-2203
title: "Oracle grid for EXP's overflow and underflow edges (T-1944)"
milestone: M5
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A dense excel-graph grid of worksheet EXP values at both ends of its range: the last finite argument before `#NUM!`
near 709.78, and the switch from 2^-1074 to 0 near -745.13. T-1944 then checks the x87 model V6 at the edges against
evidence instead of probing first.

## Context
- T-1944 (M5, functions-math, opus) is "Probe EXP's overflow and underflow edges", and its probe step is the oracle work.
  The functions lanes are full; this task fills the grid in the research lane, and T-1944 keeps the engine comparison
  and any fix.
- docs/excel-semantics/numbers.md "EXP is an x87 2^(a*log2 e) evaluation, not correctly rounded (T-1778)", paragraph
  "Worksheet EXP (T-1931)": V6 keeps a subnormal (probes/subnormal-function-results C4, C6) and overflow is `#NUM!`.
  "Not probed: the overflow edge of EXP itself, a in (709.7827, 709.79)."
- excel-graph reports a kept subnormal as 0. Every underflow cell needs a `=(EXP(A<n>))=0` companion in the same case
  (decision 0042), or the harness cannot score it.
- Layout: as probes-numeric `t2178-hyperbolic-unit-sweep` (stored inputs in column A, formulas reading them), generated
  by a short seeded script recorded in the case description. Engine model: `x87_exp::excel_exp`
  (crates/rustsheet-engine/src/x87_exp.rs).

## Acceptance criteria
- [ ] New case `eval/suites/probes-numeric/cases/t2203-exp-range-edges.case.json` (`rustsheet-case/v1`, sheet `Calc`, one
      `M5` tag). Column A holds the stored inputs, B holds `=EXP(A<n>)`, and C holds `=(EXP(A<n>))=0` for every
      underflow row:
      - overflow: 200 consecutive doubles centred on ln(DBL_MAX) = 709.782712893384, and 50 points spread over
        [709.78, 709.79], plus 709.79, 709.8, 710 and 710.5;
      - underflow: 200 consecutive doubles centred on -1075*ln 2 = -745.1332 (a result of 2^-1075, the midpoint
        between 0 and 2^-1074), 50 points spread over [-745.2, -745.0], plus -745, -745.5 and -746;
      - a control set of 20 arguments in [-708.39, -707] (normal results just above DBL_MIN = e^-708.3964).
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`),
      with no pending cell.
- [ ] numbers.md "Worksheet EXP (T-1931)" replaces "Not probed: the overflow edge" with the last finite argument and the
      first underflow-to-0 argument Excel shows. It cites the case and says how many cells the current engine matches
      (`make eval-show CASE=probes-numeric/t2203-exp-range-edges.case.json`).
- [ ] T-1944's Context names the case as its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing EXP or V6 (T-1944).
- SINH/COSH overflow (T-2169), and the cached 0 of ironcalc-calc `EXP_SIGN.xlsx` B44 (T-2204).
