---
id: T-0191
title: "Score accepted-deviation cells apart from bit-exact (decision 0036)"
milestone: M1
lane: harness
status: in-progress
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
