---
id: T-2455
title: "TRANSPOSE of a single value returns that value (not a 1x1 array)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
TRANSPOSE of a single value (scalar or one cell) returns that value, not a 1x1 array, so a lifting function over it does not lift.

## Context
Evidence: probes/t2414-gen-m7-unexplained A291, A301 and docs/excel-semantics/text.md "TRANSPOSE of a single value is that value". `COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))` is 2 in Excel, 1 in ours. `ISREF(TRANSPOSE(cell))` is FALSE.

Owned gen-m7 cells (T-2414):
- f7e94a817e5385ac A2882 `=TEXTSPLIT(TRANSPOSE(SUM(Data!B6,Data!A9:A9)),Data!A3:C7)`: B2882 Excel "", ours <empty>

## Acceptance criteria
- [x] probes/t2414-gen-m7-unexplained cells A291 and A301 pass; lines leave eval/baseline/probes.txt.
- [x] Visible parity: gen-m7/f7e94a817e5385ac B2882 (Excel "", ours empty) passes and its line leaves eval/baseline/gen-m7.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.

## Scope widened (T-2455 resume)
Size S -> M, tier sonnet -> opus (handed off). Returning the element made gen-m9 cells that matched only through the 1x1
array regress, and `make eval-baseline` refuses regressions, so each one was resolved here from probes/t2455-transpose-single-value
rather than split out: F.TEST (320d51765af9b6b7 A376, bbdfc06cae0e0d71 A34), LINEST/LOGEST (4e55500bb8ad3cd8 A178 and the
first-run order regressions, e.g. 05d44f6dd6cfc567 A195), QUARTILE (899f9555e67da2d6 A344), DISC/RECEIVED basis
(ba4ddb817a8e700e A345), and the ACCRINT, ACCRINTM and AMORDEGRC booleans (0a614cfe974b59c3 A265, 165db58d1540f014 A141,
7c20351562a2d71d A212). Each fix is limited to what its probe cells showed.

## Outcome
`fn_transpose` returns the element itself for a 1x1 result unless the argument is an array constant. That removed accidental
matches through the 1x1 array, and probes/t2455-transpose-single-value (69 cells, excel-graph) showed what Excel does with
those cells' real shapes, so this task also fixed them:
- F.TEST takes a single value as a sample of one (`#DIV/0!`) and a one-cell reference to a blank cell is `#VALUE!` ahead of
  the other sample's error (`f_test_sample`, `is_blank_one_cell`).
- LINEST and LOGEST report an argument that is itself an error value first, in argument order, and only then the data checks
  (error elements, too few points, sizes, text); a computed blank known_y is `#VALUE!` at once (`read_regression_inputs`,
  `validate_regression_inputs`, `read_matrix`).
- QUARTILE reports quart's error before the empty-data `#NUM!`.
- DISC/RECEIVED (the shared `discount_args`) read basis first; a boolean ACCRINTM par, ACCRINT issue or rate, and
  AMORLINC/AMORDEGRC salvage or rate is `#VALUE!`, after a date past the calendar's `#NUM!`.
Quirks recorded in docs/excel-semantics/text.md, statistics.md and financial.md.
Eval delta: gen-m9 known failures 8693 -> 8199 (494 fixed), gen-m7 45 -> 44, probes 2 fixed (+4 new-coverage cells of the
new probe, LINEST text const, follow-up T-2556); 0 regressions; visible parity 8139 off (lock 8632).
