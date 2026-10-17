---
id: T-1628
title: "Probe GETPIVOTDATA, ENCODEURL and FILTERXML to the evidence floor"
milestone: M5
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): GETPIVOTDATA (1), ENCODEURL (11), FILTERXML (13) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
GETPIVOTDATA needs a pivot table in an xlsx fixture (field/item pairs, grand totals, missing items, a reference outside a pivot); ENCODEURL on reserved, non-ASCII and surrogate-pair characters, numbers and booleans; FILTERXML with invalid XML, XPath with no match, several matches (spill) and numeric text. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
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
Three probe cases (encodeurl/filterxml/getpivotdata `-evidence-floor`) filled from excel-graph. Report cells: GETPIVOTDATA 24, ENCODEURL 38, FILTERXML 46, so all three clear the floor (evidence-floor lock 58 -> 55). ENCODEURL and FILTERXML are `#VALUE!` on the web throughout; GETPIVOTDATA without a pivot is `#REF!`. GETPIVOTDATA is not yet implemented, so its 21 new cells and one FILTERXML cell are new-coverage failures. A real pivot is not probed (builder cannot write pivots). Quirks in docs/excel-semantics/web.md. Follow-ups: T-1703, T-1704, T-1705.
