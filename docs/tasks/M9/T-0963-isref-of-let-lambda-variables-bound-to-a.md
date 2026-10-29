---
id: T-0963
title: "ISREF of LET/LAMBDA variables bound to a reference (ISREF.xlsx A19-A26)"
milestone: M9
lane: functions-logic
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-0965]
---

## Goal
`ISREF(x)` is TRUE inside LET/LAMBDA when the variable was bound to a reference, as Excel does.

## Context
Excel-cached cells in ironcalc-calc/INFORMATION/ISREF.xlsx Sheet1!A19, A20, A23, A24, A26 (`LAMBDA(x,ISREF(x))(K1)`, `LET(x,K1,ISREF(x))`, `LET(x,TextCell,ISREF(x))`) are TRUE but we return FALSE: `LET` evaluates the binding with `evaluate_node_in_context` (functions/logical/let.rs) and lambda arguments likewise, so the reference is lost. Upstream: the `variable_value`/"a variable holds what it was bound to, a reference included" change around `c4641f0f` (information/mod.rs, model.rs). Found by T-0940.

Update 2026-10-07: T-0223 landed the code (LET/LAMBDA bind references, `fn_isref` of a variable;
MIN_MAX_extras.xlsx A14-A16 pass). What remains is confirming the five ISREF.xlsx cells once T-0965 adds
the workbook; this may then shrink to a check-and-close.

## Acceptance criteria
- [ ] The five ISREF.xlsx cells above pass; `ISREF(x)` for a variable bound to a value is still FALSE.
- [ ] 0 eval regressions, `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
