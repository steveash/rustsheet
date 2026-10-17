---
id: T-1625
title: "Probe VDB, AMORDEGRC, FVSCHEDULE, MDURATION, RRI and IMPOWER, IMSUB, IMDIV to the evidence floor"
milestone: M9
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): VDB (10), AMORDEGRC (13), FVSCHEDULE (13), MDURATION (13), RRI (17), IMPOWER (6), IMSUB (14), IMDIV (17) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
VDB fractional periods and no_switch, AMORDEGRC life bands and rate thresholds and basis, FVSCHEDULE with blanks/text in the schedule, MDURATION frequency and basis, RRI with zero/negative pv/fv, IM* with i vs. j suffixes, mixed suffixes, zero divisor, non-integer powers and text that is not a complex number. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).

## Outcome
Added probes fin-vdb-modes, fin-rri, fin-fvschedule, fin-amordegrc, fin-mduration, eng-impower, eng-imsub,
eng-imdiv (excel-graph filled). All eight functions are over the 20-cell floor (below_floor 21 -> 13).
44 new-coverage failures baselined, 0 regressions. Quirks in docs/excel-semantics/{financial,engineering}.md;
fixes filed as T-1764, T-1765, T-1766.
