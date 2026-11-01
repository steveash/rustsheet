---
id: T-3293
title: "TEXTJOIN ignore_empty FALSE: probe a limit crossed inside the formula's own row, stop mid-row if Excel does"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A whole-row TEXTJOIN with ignore_empty FALSE whose 32,767-unit limit is passed partway along the formula's own row gives what Excel gives (`#CALC!` if Excel stops reading mid-row, circular otherwise).

## Context
Follow-up of T-3123 (excel-fidelity review MINOR). `textjoin_push_padded_range` (functions/text/concatenate.rs) walks one row at a time and stops at row boundaries, but within a row `textjoin_push_range` still reads the rest of the row's cells after `over_limit`. Excel stops reading past the limit (probes/textjoin-whole-rows-self-reference and -full-width B5; docs/excel-semantics/text.md), but no probe crosses the limit inside the TEXTJOIN cell's own row, e.g. a long text in A5 and `=TEXTJOIN(",",FALSE,1:10)` in Z5, or the limit passed by padding left of the formula cell.

## Acceptance criteria
- [x] A probe (excel-graph) pins a self-including whole-row TEXTJOIN whose limit is crossed before the formula cell in the formula's own row.
- [x] The engine matches it (stop mid-row if Excel does); unit test added; text.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/t3293-textjoin-own-row-limit (excel-graph): Excel gives #CALC! when the limit is passed before the formula cell in its own row (ZZ1, ZZ9) and circular otherwise. `textjoin_push_range` now stops its walk once over the limit; unit test and text.md added. Eval: 11/11 probe cells pass, no baseline additions, regressions 0.
