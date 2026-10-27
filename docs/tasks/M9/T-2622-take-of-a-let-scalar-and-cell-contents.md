---
id: T-2622
title: "TAKE of a LET scalar, CELL(contents) in COUNT and MODE.SNGL, XMATCH, and a LAMBDA joined as text is #VALUE! (T-2475 probe)"
milestone: M9
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
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
- [x] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence. Except: probes/t2475 P!A279 (a one-cell TAKE cut minus a scalar error is a scalar error; ours a 1x1 array) still fails and moves to T-2857; the new probe's A6 and A18 are known failures owned by T-2630.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).

## Outcome
Probe `probes/t2622-contents-xmatch-take-lambda` (excel-graph) found five separate rules, all recorded in docs/excel-semantics (information.md, lookup.md, statistics.md, arrays.md): CELL("contents") of a blank counts in COUNT/COUNTA and is `#VALUE!` in MODE; XMATCH match_mode 3 takes a number as its text; TAKE/DROP of a one-cell reference stays a reference (TYPE 1, `#VALUE!` for a blank in PEARSON-family); an error inside an array data argument of PERCENTILE/QUARTILE/PERCENTRANK is that error, after the scalar arguments' errors; an uncalled LAMBDA joined as text or compared is `#VALUE!`. Eval against `main` at the 2026-10-27 review (after main merges): gen-m9 -3 lines (92d4fa29e5510f1e A98, c2684066d8df263e A201 and 1b31092a0d79e5f1 A143; the rest of the owned cells had already passed on `main`), probes -11 t2475 lines and +2 new known failures in the new probe (t2622 Calc!A6, A18, T-2630), no regressions. Not fixed: P!A279 (T-2857).
Not done: CELL("contents") wrapped in IF counted in COUNT / MODE (Excel does; follow-up).

Review fixes (2026-10-27, steward REQUEST_CHANGES of 2026-10-26):
- The PERCENTILE-family rule: this branch's own `DataError::AfterScalars` was dropped when `main` was merged in (c7e6241b),
  because `main` had meanwhile implemented the same rule generally (statistics.md "An error element of an array argument
  is that error (T-2255)": `numeric_flat_with_element_errors`, `array_element_error`, `lift_over_sorted`). Every probe cell
  of that rule passes on it, so nothing is restored. statistics.md now names that code, and the unit test
  `percentile_family_array_error_element_order` pins the probed order.
- XMATCH match_mode 3: `main` already behaved this way (T-2669's `regex_pattern`); this task adds the probe and a test.
  lookup.md says so.
- COUNT and COUNTA share one helper for a `CELL("contents", ..)` argument (`walk_cell_contents_arg`), and the text and
  number uses of an uncalled LAMBDA share `lambda_value_error`. arrays.md's `Implemented in:` names (`cast_string_ref`,
  `handle_comparison`) were checked.
- Still failing and owned by T-2630: A6 (`COUNT(IF(TRUE,CELL("contents",C5)))`) and A18 (`MODE.SNGL(IF(TRUE,...))`).
