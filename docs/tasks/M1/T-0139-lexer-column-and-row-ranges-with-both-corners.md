---
id: T-0139
title: "Lexer: column and row ranges with both corners sheet-qualified (Data!A:Data!B)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`Data!A:Data!B`, `Data!1:Data!2` and `Data!A1:Data!B` parse as ranges, as in Excel (very likely; probe first).

## Context
Found by the T-0130 fidelity review. T-0130 handles only `Data!B8:Data!E12`; these forms still give `#ERROR!`. Code: `expressions/lexer/ranges.rs`. Use the `excel-probe` skill for oracle values; note sheet-name matching is case-sensitive in the engine (Excel is case-insensitive), file a separate task if none exists.

## Acceptance criteria
- [ ] Oracle-backed probe cases for the three forms; each parses and evaluates like the single-prefix form.
- [ ] references.md entry updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/sheet-qualified-both-corners` (oracle: excel-graph) shows the premise was wrong: with
column or row corners a second sheet prefix is not accepted. `Data!A:Data!B` and its variants give
`#NAME?`; `Data!1:Data!2`, `Data!A1:Data!2` and `Data!$A:Data!$B` make the workbook unloadable.
Recorded in references.md. Parsing them as ranges would be wrong, so no lexer change. The engine's
`#ERROR!` vs Excel's `#NAME?` is filed as T-0145. Eval delta: 6 new known failures (new coverage), 3 new passes.
