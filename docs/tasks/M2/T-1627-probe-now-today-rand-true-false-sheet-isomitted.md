---
id: T-1627
title: "Probe NOW, TODAY, RAND, TRUE, FALSE, SHEET, ISOMITTED, TOROW, WRAPCOLS, WRAPROWS to the evidence floor"
milestone: M2
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): NOW (5), TODAY (6), RAND (4), TRUE (6), FALSE (8), SHEET (16), ISOMITTED (17), TOROW (14), WRAPCOLS (16), WRAPROWS (16) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
volatile results checked by shape-stable formulas (INT(NOW())=TODAY(), RAND() in [0,1), ISNUMBER, arithmetic with them), TRUE()/FALSE() with arguments, SHEET with references, names, text and no argument, ISOMITTED inside LAMBDA with omitted/blank/supplied arguments, TOROW ignore modes 0-3 and scan_by_column, WRAP* wrap_count 0/1/larger than the vector and pad_with. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [x] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [x] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [x] Quirks recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).

## Outcome
Three probe cases (probes/volatile-now-today-shapes 48 cells, volatile-and-info-rand-true-false-sheet 108, dynamic-array-reshape-and-isomitted 132; excel-graph, 2026-10-15) lift all ten functions over the floor (quality/evidence-floor.json 73 -> 63). 21 new-coverage failures, all WRAPROWS/WRAPCOLS (T-1630) or SHEET (T-1631); NOW, TODAY, RAND, TRUE, FALSE, ISOMITTED and TOROW match. Quirks in arrays.md and information.md.
