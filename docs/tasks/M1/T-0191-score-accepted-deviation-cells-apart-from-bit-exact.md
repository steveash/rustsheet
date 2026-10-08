---
id: T-0191
title: "Score accepted-deviation cells apart from bit-exact (decision 0036)"
milestone: M1
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Cells that differ from Excel by design (an accepted `DEVIATION`, decision 0014) are listed per suite, scored in their own `deviations` column with their by-design value pinned, and excluded from bit-exact. probes/resource-limits C12/C14 move there, so M1's "100% bit-exact" is measurable (decision 0036).

## Context
Decision 0036 sets out the file format (`<case>\t<Sheet!A1>\t<decision NNNN>\t<semantics doc#anchor>\t<pinned value>`, modelled on decision 0032 `cached_overrides`, crates/rustsheet-eval/src/overrides.rs) and the scoring: pinned value gives deviation, Excel value gives pass plus a stale-line warning, anything else gives failure. First cells: probes/lifted-range-rows C1 `=ROWS(DATE(11:1899,12,1))`, C2 `=COLUMNS(DATE(11:1899,12,1))` C3 `=DATE(11:1899,12,1)` and E1 `=INDEX(DATE(11:1899,12,1),5,7)`, pinned `#ERROR!` (docs/excel-semantics/arrays.md, "Array results above 1,048,576 cells are #ERROR!"; resource-limits C12 and C14 pass since T-0047 raised the bound to one whole column).

## Acceptance criteria
- [ ] `rustsheet-eval` reads an optional `deviations` file per suite. The case/cell must exist, the cited semantics entry must have a `Status: DEVIATION` line, and the pinned value must parse; otherwise it is a typed error, never a panic.
- [ ] Scoring follows decision 0036. Deviation cells are not in the bit-exact denominator or in baseline F lines, and the report has a per-suite `deviations` column.
- [ ] `eval/suites/probes/deviations` lists C12 and C14. `make eval-baseline` removes their F lines (no hand edit), and probes reaches 100% of non-deviation cells if no other failures remain.
- [ ] Unit tests cover parse errors and the three scoring outcomes. docs/process/quality-gates.md and the steward skill mention the file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any limit; adding deviation lines beyond C12/C14 (each needs its own evidence).

## Outcome
- `rustsheet-eval` reads the suite manifest key `deviations = "<file>"` (crates/rustsheet-eval/src/deviations.rs).
  Malformed lines, missing cases and cited entries without `Status: DEVIATION` are a typed
  `DeviationError`, checked by `eval` and `eval-check` before any case runs. A line naming an
  unscored cell makes the case invalid.
- Scoring follows decision 0036. The pinned value gives the new `deviation` verdict, which leaves
  the bit-exact rate and is never an F line. Excel's value passes with a stale-line warning in
  the CLI and the report. Anything else keeps its mismatch verdict, so it fails the gate. The
  report and the CLI summary have a per-suite `deviations` column.
- The listed cells are probes/lifted-range-rows C1, C2, C3 and E1, not resource-limits C12/C14,
  which have passed since T-0047, as the Context notes. Decision 0036 records this and retires its
  interim rule.
- Eval delta: probes known failures 255 → 251 (4 cells now deviations, via `make eval-baseline`);
  no other suite changed and there are 0 regressions.
- Unit tests cover the parse errors, the entry check, the missing case, the three outcomes
  end-to-end, and the report rate and column. docs/process/quality-gates.md and the steward skill
  describe the file.
