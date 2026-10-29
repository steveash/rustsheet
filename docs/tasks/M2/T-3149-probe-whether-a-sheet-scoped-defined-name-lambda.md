---
id: T-3149
title: "Probe whether a sheet-scoped defined-name LAMBDA snaps its body's near-zero root, as a workbook-scoped one does"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know from Excel whether a LAMBDA bound to a sheet-scoped defined name snaps its body's near-zero `+`/`-` root when called,
and make the engine agree.

## Context
- probes/t3094-snap-lambda-call-forms (excel-graph 2026-10-29) shows a workbook-scoped named LAMBDA snaps its body's root
  (docs/excel-semantics/numbers.md "Near-zero snapping", paragraph "Pickers, LET, LAMBDA and unary signs"). Sheet-scoped
  names were out of scope there and in T-3105.
- Since T-3105, `Model::call_lambda` (functions/logical/lambda.rs) snaps every direct call, so a sheet-scoped name snaps
  too, by extension only.
- Method: the `excel-probe` skill. Copy t3094's rows 2-5 (`x+0.2-0.3` at 0.1, k = 7, k = 8, `-x-0.2+0.3`) into a case
  whose `f` is scoped to the calling sheet, called as `=f(..)`, `=f(..)+0` and `=SUM(f(..))`, plus a grouped-body control.
  Check first how the case format declares a sheet-scoped name (`rg -n '"scope"' eval/suites/probes/cases | head`).

## Acceptance criteria
- [ ] A new oracle-filled probe case covers the sheet-scoped calls; every cell passes (engine change only if Excel differs).
- [ ] numbers.md records the result (VERIFIED) and drops the "by extension" note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA helpers (T-3095).
