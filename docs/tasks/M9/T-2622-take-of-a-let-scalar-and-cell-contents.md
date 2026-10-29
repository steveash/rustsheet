---
id: T-2622
title: "TAKE of a LET scalar, CELL(contents) in COUNT and MODE.SNGL, XMATCH, and a LAMBDA joined as text is #VALUE! (T-2475 probe)"
milestone: M9
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
TAKE of a LET-bound scalar is that scalar, CELL("contents") reads as Excel does inside COUNT and MODE.SNGL, and XMATCH's match mode cell is read as Excel does.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (5; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/1b31092a0d79e5f1.case.json Calc!A143: probe P!A70 `TAKE(LET(v,D05!A$11,acc,v,y,"?b",acc),-1,-1)`: Excel 0.0, ours "64|1x1|0"
- gen-m9/775d053de498b8c7.case.json Calc!A235: probe P!A279 `(TAKE(D20!C8:E12,D20!B12,D20!$A7)-(D20!A$3/D20!A2))`: Excel "#2", ours "64|1x1|#2"
- gen-m9/92d4fa29e5510f1e.case.json Calc!A98: probe P!A326 `MODE.SNGL(CELL("contents",D26!C$5))`: Excel "#3", ours "#7"
- gen-m9/c2684066d8df263e.case.json Calc!A201: probe P!A491 `COUNT(D38!B2:B10,CELL("contents",D38!C8))`: Excel 3.0, ours 2.0
- gen-m9/eccd03df42e40006.case.json Calc!A21: probe P!A549 `XMATCH(COUNTA(D45!B8),{3;1;4;1;5},(D45!$C1+D45!E8))`: Excel 2.0, ours "#7"

Also: a LET-bound uncalled LAMBDA joined as text (`t&"|"&v`) is `#VALUE!` in Excel, ours `#N/IMPL!` (the probe's bare `LAMBDA(..)` rows, e.g. P!A1 of each LAMBDA cell; arrays.md "An uncalled LAMBDA is one value").

The LAMBDA-as-text rule is inferred from the wrapper (`t&"|"&v` with TYPE 128); add a direct probe cell (`LAMBDA(x,x)&""`)
before implementing it.

## Acceptance criteria
- [x] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).

## Handoff (resumed)
Tried: read the task and the existing "empty value" handling (`CELL("contents",<blank>)` is the empty value, special-cased in
functions/information/mod.rs ~788, drop_take.rs, hstack_vstack.rs, spill_functions.rs ~282; information.md "CELL("contents", ref)
as a VSTACK argument"). No code changed. Why handed off: the five owned cells need five distinct rules (TAKE of a LET scalar,
CELL("contents") as COUNT's and MODE.SNGL's argument, XMATCH match-mode cell, LAMBDA joined as text) whose evidence is inferred from
a wrapper probe, and the task itself says the LAMBDA-as-text rule needs a direct probe cell first (`LAMBDA(x,x)&""`). That is a
probe-then-design job across the empty-value representation, not a one-function fix.
Next: add the direct probe cells (oracle excel-graph is OK), fill them, record each rule in docs/excel-semantics/, then fix the
empty-value reads in COUNT / MODE.SNGL, TAKE's scalar path and XMATCH's match_mode coercion, and `make eval-baseline`.

## Outcome
Four of the five owned cells (Calc!A235, A98, A201, A21: TAKE of a range, CELL("contents") in MODE.SNGL and COUNT, XMATCH's regex
match mode 3) already passed on main when this was resumed (fixed by T-3072, T-2669 and the CELL("contents") work). The remaining two
rules got a direct probe, probes/t2622-take-let-reference-lambda-text (40 cells, excel-graph, all pass):
- a LET variable or LAMBDA parameter bound to a one-cell reference stays that reference as TAKE's, DROP's or TRIMRANGE's array (bare,
  as a LET body, as an IF arm): `Model::cell_variable_reference`, read by `value_or_grid(.., keep_cell_variables)` and
  `evaluate_picked_source(.., keep_cell_variables)`; lookup.md "TAKE/DROP of a variable bound to one cell is a reference".
- an uncalled LAMBDA joined with `&` is `#VALUE!` (was `#N/IMPL!`): `handle_concatenate`; arrays.md "An uncalled LAMBDA is one value".
Eval delta: gen-m9 known failures 3305 -> 3304 (1b31092a0d79e5f1 Calc!A143), probes 2977 -> 2969 (8 t2475-stat-subexpressions cells)
plus the new 40-cell case passing; no regressions.
