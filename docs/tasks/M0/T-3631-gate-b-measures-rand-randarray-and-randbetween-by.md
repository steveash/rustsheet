---
id: T-3631
title: "Gate B measures RAND, RANDARRAY and RANDBETWEEN by their semantic probes, not the 99.9% rate (decision record)"
milestone: M0
lane: harness
status: todo
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
- [ ] A new probe case under eval/suites/probes/cases/ (milestone tags per decision 0057), filled by
      excel-graph, with at least 20 invariant cells per function: for example `RANDBETWEEN(5,5)`,
      `RANDBETWEEN(-3,-3)`, `ROWS(RANDARRAY(3,2))`, `COLUMNS(RANDARRAY(3,2))`,
      `AND(RAND()>=0,RAND()<1)`, `INT(RANDBETWEEN(1,9))=RANDBETWEEN(1,9)`-style integer checks with one
      draw, `RANDARRAY(2,2,1,1,TRUE)`, `RANDARRAY(,,7,7)`, coerced arguments (text numbers, booleans)
      and argument errors (`RANDBETWEEN(2,1)`, `RANDARRAY(0)`, `RANDARRAY(-1)`, non-integer bounds).
      Every cell's value must not depend on the draw; the case says why for each form.
- [ ] The eval report counts those cells for the three functions: `v15_checkpoint` and `gate_b` show
      each with ≥ 20 cells; if attribution drops them (sole call), fix it for these three functions
      only, as decision 0225 says.
- [ ] Any cell where the engine differs from Excel is fixed in this task or owned by a new task.
- [ ] Quirks found go into docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Comparing drawn values with Excel, and any distribution test of the draws.
- Other volatile functions (NOW, TODAY, OFFSET, INDIRECT, CELL, INFO).
