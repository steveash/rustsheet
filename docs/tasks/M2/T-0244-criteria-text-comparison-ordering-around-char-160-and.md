---
id: T-0244
title: "Criteria text comparison ordering around CHAR(160) and spaces (probes/criteria-padded-text C13)"
milestone: M2
lane: functions-criteria
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
COUNTIF text comparison operators (`>`, `<`) order strings around CHAR(160) and spaces exactly as Excel does.

## Context
probes/criteria-padded-text C13 (`">"&CHAR(160)&"0"` over text cells "1","1 "," 1",CHAR(160)&"1","1"&CHAR(160)) is 4 in Excel, 1 in the engine, which compares raw lowercase strings in `result_is_greater_than_string` (functions/criteria.rs). Needs more probes to find Excel's collation, so tier opus. See docs/excel-semantics/criteria.md.

## Acceptance criteria
- [x] probes/criteria-padded-text C13 passes and the baseline entry is removed; collation rule documented.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed text ordering around CHAR(160) and spaces (probes/text-order-nbsp-space, 333 cells,
excel-graph): character order with CHAR(160) right after the space, before digits and letters, in
`>` and COUNTIF alike. New `functions::util::cmp_text_excel` is used by `compare_values` and the
four text-ordering criteria. probes/criteria-padded-text C13 now passes (4). Eval: C13 fixed, new
case 332/333 (the hyphen cell, `"-1" > "0"`, is left to T-0220). Rule in coercion.md and criteria.md.
