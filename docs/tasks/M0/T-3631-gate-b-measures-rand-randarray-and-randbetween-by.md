---
id: T-3631
title: "Gate B measures RAND, RANDARRAY and RANDBETWEEN by their semantic probes, not the 99.9% rate (decision record)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
RAND, RANDARRAY and RANDBETWEEN meet Gate B (decision 0188 R1(b)) on an oracle-filled probe of their
invariant cells, per decision 0225, so Gate B can reach 506/506.

## Context
- Decision 0225 (owner ruling 2026-11-01): the three functions are read on their invariant cells
  (values that do not depend on the draw); each meets the rate with at least 20 scored cells, all
  matching Excel.
- The eval run of 2026-11-01 (b6c931bd2) gives all three 0 attributed and 0 calling cells, because
  volatile-dependent cells are skipped (T-0005). The harness already scores invariant cells
  (`crates/rustsheet-eval/src/volatile.rs`, `invariant_cells_are_scored_and_stop_the_taint`), so the
  likely gap is only the missing probe; check whether attribution (`v15_checkpoint`, sole call / every
  call) also needs a change.

## Acceptance criteria
- [x] A new probe case under eval/suites/probes/cases/ (milestone tags per decision 0057), filled by
      excel-graph, with at least 20 invariant cells per function: for example `RANDBETWEEN(5,5)`,
      `RANDBETWEEN(-3,-3)`, `ROWS(RANDARRAY(3,2))`, `COLUMNS(RANDARRAY(3,2))`,
      `AND(RAND()>=0,RAND()<1)`, `INT(RANDBETWEEN(1,9))=RANDBETWEEN(1,9)`-style integer checks with one
      draw, `RANDARRAY(2,2,1,1,TRUE)`, `RANDARRAY(,,7,7)`, coerced arguments (text numbers, booleans)
      and argument errors (`RANDBETWEEN(2,1)`, `RANDARRAY(0)`, `RANDARRAY(-1)`, non-integer bounds).
      Every cell's value must not depend on the draw; the case says why for each form.
- [x] The eval report counts those cells for the three functions: `v15_checkpoint` and `gate_b` show
      each with ≥ 20 cells; if attribution drops them (sole call), fix it for these three functions
      only, as decision 0225 says.
- [x] Any cell where the engine differs from Excel is fixed in this task or owned by a new task.
- [x] Quirks found go into docs/excel-semantics/.
- [x] `make check` green.
- [ ] Reviewers approve.

## Out of scope
- Comparing drawn values with Excel, and any distribution test of the draws.
- Other volatile functions (NOW, TODAY, OFFSET, INDIRECT, CELL, INFO).

## Outcome
- Probe, split by milestone (decision 0057): probes/t3631-rand-randbetween-invariant (M5; RAND R!A1:A30,
  RANDBETWEEN R!B1:B31) and probes/t3631-randarray-invariant (M7; RANDARRAY A!A1:A36, B1:B5 shape
  readers, D1:E2 spill): 30 + 31 + 42 listed invariant cells, 106 scored with the three spill
  members, filled by excel-graph twice with no cell varying; the descriptions say why each form does
  not depend on the draw. Review round 1: `RANDBETWEEN(1E15,1E15)` became MAX and MIN over 1000
  draws (R!B24, B31; both exactly 1E15, so Excel's draw never leaves the bounds where the double
  spacing is 0.125), and A!A31:A36 check min = max RANDARRAY (0.5, -2, "1.5") over 1000 draws (all
  exact, so A6, A21 and A30 stay). The engine matches all 106; new quirks (RANDARRAY takes boolean
  bounds and counts, RANDBETWEEN padded number text, more CEILING/FLOOR points, no escape at 1E15)
  are in docs/excel-semantics/numbers.md.
- Attribution needed no change (sole call vs every call): every function's sole-call cells are
  above the evidence floor. The gap was the suite filter: Gate B never read the probes suite. Now
  `gate_b::summarize` reads probes cells toward RAND, RANDARRAY and RANDBETWEEN only (decision
  0225), via `report::attribute_cell_to` (`attribute_cell` delegates to it); test
  `probe_cells_count_toward_the_random_functions_only`, and `only_the_three_random_functions_are_read_on_probes`
  pins the list (a wider one needs its own decision record).
- Eval (local, calipers/poi/corpus-bench not fetched, so Gate B overall reads n/a): Gate B rows
  RAND 38/38, RANDARRAY 53/53, RANDBETWEEN 52/52 cells within 15 digits, all meet the rate (were
  0 cells); v15_checkpoint the same, attribution `sole`. Baseline: 2 new probe cases (61 and 45
  cells), 0 known failures, 0 regressions.
