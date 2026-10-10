---
id: T-1283
title: "Score run-invariant probe cells that call NOW, TODAY, RAND, RANDBETWEEN or RANDARRAY (no scorable Excel evidence today)"
milestone: M2
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A probe case can mark cells whose formulas call a volatile function but whose value does not change between runs (for example `=RANDBETWEEN(5,5)`, `=ISNUMBER(NOW())` or `=RANDBETWEEN(2,1)`), and the eval harness scores those cells against the oracle. NOW, TODAY, RAND, RANDBETWEEN and RANDARRAY then get Excel evidence for their argument checks, error kinds, result types and shapes.

## Context
Gap triage 2026-10-09 (decision 0031): report.md "Excel evidence per function" lists RAND, RANDARRAY and RANDBETWEEN (M5) as "Implemented, no Excel evidence", and NOW (2 Excel cells) and TODAY (1), both M2, as below the 25-cell threshold. A new probe cannot close this gap. `crates/rustsheet-eval/src/volatile.rs` (`is_volatile`, `scorable_cells`) skips every cell whose formula, or any precedent of it, calls RAND/RANDBETWEEN/RANDARRAY, and also NOW/TODAY unless the workbook pins the clock through a `METADATA` sheet (`pinned_clock_serial`, IronCalc's mock-time convention, which excel-graph cannot honour). The skip covers deterministic answers too: `RANDBETWEEN(5,5)` = 5, `RANDBETWEEN(2,1)` (an error), `ROWS(RANDARRAY(3,2))` = 3, `RANDARRAY(1,1,5,5,TRUE)` = 5, `RANDARRAY(0)` (an error), `AND(RAND()>=0,RAND()<1)`, `ISNUMBER(NOW())`, `TODAY()=INT(NOW())`, `NOW(1)` (a load error?). docs/architecture/eval-harness.md "Scoring" documents the skip.

Design to decide (opus): a per-case opt-in list of invariant cells in `rustsheet-case/v1` (or a per-cell flag), the check that a listed cell gave the same Excel value on two fills (or a rule that the oracle fill is repeated), and how a listed cell that turns out to vary is reported. A cell is listed only when its value cannot change between runs. Keep the default skip for everything else. Update eval-harness.md and, if the case format changes, docs/architecture's case-format description. Then add one probe case for the five functions (excel-probe skill), with the cells above plus argument coercion (`RANDBETWEEN("5","5")`, `RANDBETWEEN(5.5,5.7)`, `RANDARRAY(2,2,1,1)`, `RANDARRAY(,,,,"x")`), and fill it on excel-graph.

## Acceptance criteria
- [ ] rustsheet-eval scores the opted-in cells of a case even though they call a volatile function, and still skips every other volatile cell (unit tests in volatile.rs for both, and for a precedent that is opted in).
- [ ] A probe case for NOW, TODAY, RAND, RANDBETWEEN and RANDARRAY is committed with its excel-graph sidecar, and report.md's evidence table no longer lists RAND, RANDARRAY or RANDBETWEEN as implemented with no Excel evidence. Failing cells are baselined as new coverage and filed as tasks.
- [ ] eval-harness.md "Scoring" describes the opt-in; `make check` green; reviewers approve.

## Out of scope
- Statistical tests of the random distributions (uniformity), and the engine's RNG seeding.
