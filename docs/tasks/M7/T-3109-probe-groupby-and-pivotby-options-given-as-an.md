---
id: T-3109
title: "Probe GROUPBY and PIVOTBY options given as an OFFSET, CHOOSE or IF reference and PIVOTBY's field_headers as an INDEX reference"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence that a GROUPBY/PIVOTBY option (field_headers, total depths) given as a multi-cell reference returned by
OFFSET, CHOOSE or IF lifts as a written range does, and that PIVOTBY's options do the same with an INDEX reference.

## Context
- T-3088 made `Model::lift_options` (functions/lookup_and_reference/lifted_options.rs) lift over any multi-cell
  reference an option's expression returns (`Model::freeze`). Only GROUPBY's total_depth through INDEX is evidenced
  (probes/t2707-prob-orders A37:A65; docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY lift over an array
  field_headers or total depth"); other reference-returning functions and PIVOTBY are generalisations.
- Reuse Data!A4:D4 / Data!D1:E1 shapes from probes/t2707-prob-orders and probes/groupby-lifted-args via `excel-probe`.

## Acceptance criteria
- [ ] An oracle-filled probe covers GROUPBY total_depth and field_headers as `OFFSET(..)`, `CHOOSE(1,range)`,
      `IF(TRUE,range)`, and PIVOTBY field_headers/row_total_depth/col_total_depth as `INDEX(range,0)`, with ROWS/COLUMNS
      and ERROR.TYPE readings.
- [ ] arrays.md records the result; any engine mismatch has a functions-lookup follow-up with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (follow-up task).
