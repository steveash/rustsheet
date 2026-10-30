---
id: T-2969
title: "spec2xlsx writes a table column's calculatedColumnFormula; probe whether Excel refills it on load"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/ states, with excel-graph evidence, whether a workbook whose table part stores a
`calculatedColumnFormula` is filled when Excel opens it, so the engine's import (T-2958 follow-up) can match.

## Context
Found by T-2957 (docs/excel-semantics/xlsx-storage.md, "A calculated column is saved as `calculatedColumnFormula`
plus one formula per row"). Excel writes `<calculatedColumnFormula>` in the `<tableColumn>` of a filled column, in
file form (`T[[#This Row],[Col]]*2`, or `A2*2` for an A1 formula, the first data row's text), plus `dataDxfId`, and
each data cell keeps its own `<f>`. spec2xlsx cannot write a column formula today (tables take `name`, `ref`,
`filter`, `totals_row`), so whether Excel refills a column on load is unprobed.
1. Add a table option to spec2xlsx (e.g. `"calculated": {"C": "T[[#This Row],[Col]]*2"}`, keyed by column letter)
   that writes `<calculatedColumnFormula>` for that column, with a tools test pinning the written XML. Document it in
   the module docstring. The engine adapter ignores it unless the import already reads it.
2. Probe cases (tag M2, under eval/suites/probes/cases/): the column formula with (a) every data cell blank, (b) one
   data cell holding the formula and the others blank, (c) a data cell holding a constant; FORMULATEXT of each data
   cell outside the table, and a typed formula into a blank data cell after load (does it re-fill?).
3. Record the rule in xlsx-storage.md under the entry above.

## Acceptance criteria
- [ ] spec2xlsx option + test; probe cases filled on excel-graph and in the probes baseline.
- [ ] xlsx-storage.md states the load rule with the cells as evidence; an engine task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine's fill on typed entry (T-2958) and auto-expansion (T-2968).

## Outcome
spec2xlsx takes a table `"calculated": {"C": "<formula>"}` option (test in tools/tests/test_spec2xlsx_tables.py).
Probe `t2969-calculated-column-load` filled on excel-graph: Excel does not refill a calculated column on load
(rule in xlsx-storage.md). The engine does not read `calculatedColumnFormula`, so it matches the load rule; the one failing cell (FORMULATEXT shows `[@Val]` in Excel) is filed as T-3315.
