---
id: T-2223
title: "List probes/textsplit-padded-size A1-A4 as accepted directive-5 deviations (TEXTSPLIT's 2^20-cell bound)"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The four probe cells where TEXTSPLIT's documented directive-5 bound gives `#ERROR!` are scored as
accepted deviations (decision 0036), not as known failures. M7's probe exit criterion then counts only
real gaps.

## Context
- T-1781 (done) bounded TEXTSPLIT's padded result with `limits::check_array_cells`. Its Outcome says
  "4 new deliberate known failures (DEVIATION, probe textsplit-padded-size A1-A4)". The bound is part of
  the arrays.md DEVIATION entry "Array results above 1,048,576 cells are #ERROR!" (Status: DEVIATION
  (decision 0014)), which describes the TEXTSPLIT case and these cells.
- The cells (excel-graph):
  - `ROWS(...)` and `COLUMNS(TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";"))` are 1501 in Excel
    (A1, A2).
  - The 1024 + 1024 delimiter text gives 1025 (A3, A4).
  - The engine gives `#ERROR!` for all four.
- The four cells are still F lines in eval/baseline/probes.txt, so the M7 row of the report's "Probe
  milestones (decision 0051)" counts them as known failures (gap triage 2026-10-21, `main` fce6020d).
- Precedent: probes/lifted-range-rows C1-C3 and E1 are already in `eval/suites/probes/deviations`,
  citing `docs/excel-semantics/arrays.md#array-results-above-1048576-cells-are-error`, pinned
  `#ERROR!`.
- Protected change (decision 0036, scripts/check_governance.py `is_deviations_file`). The PR needs a
  decision record in docs/decisions/ (`make new-decision`), and the Excel-fidelity reviewer confirms the
  cited limit is a directive-5 allocation bound, not a convenience.

## Acceptance criteria
- [x] `eval/suites/probes/deviations` has four new lines for
      `textsplit-padded-size.case.json` `Calc!A1`..`Calc!A4`, each citing decision 0014 and the arrays.md
      entry above, pinned `#ERROR!`.
- [x] A decision record (new, or an amendment of the one that holds the deviations list) states why these
      cells are directive-5 deviations.
- [x] `make eval-baseline` (never by hand) drops the four F lines from eval/baseline/probes.txt and adds
      no line.
- [x] `make eval` reports the four cells in the probes `deviations` column, and the M7 probe-milestone
      known failures drop by 4.
- [x] arrays.md's DEVIATION entry names the four cells as listed in eval/suites/probes/deviations,
      next to lifted-range-rows.
- [x] `make check` green; reviewers (including Excel-fidelity) approve.

## Out of scope
- probes/scalar-lift-open-c3 W1 `ROWS(HSTACK(A:A,1:1))`. Ours is `#CIRC!`, not the pinned `#ERROR!`;
  T-1965 owns it.
- Raising the bound.

## Outcome
- probes/textsplit-padded-size `Calc!A1`..`Calc!A4` are listed in eval/suites/probes/deviations (decision 0014, the
  arrays.md "Array results above 1,048,576 cells are #ERROR!" entry, pinned `#ERROR!`), under decision 0091.
- `make eval-baseline` dropped their four F lines from eval/baseline/probes.txt (probes known failures 1668 -> 1664,
  0 regressions) and visible-parity 1769 -> 1765. `make eval`: the case is gone from "Failing cases in M7".
- arrays.md names the four cells next to lifted-range-rows.
