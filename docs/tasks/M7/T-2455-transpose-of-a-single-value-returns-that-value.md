---
id: T-2455
title: "TRANSPOSE of a single value returns that value (not a 1x1 array)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
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
- [ ] probes/t2414-gen-m7-unexplained cells A291 and A301 pass; lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7/f7e94a817e5385ac B2882 (Excel "", ours empty) passes and its line leaves eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.

## Handoff
Tried: `fn_transpose` (functions/lookup_and_reference/transpose.rs) now returns the element itself for a 1x1 result unless the argument is an array literal. `make eval-baseline` with that alone: probes A291/A301 and gen-m7 f7e94a817e5385ac B2882 pass, but two gen-m9 cells regress because they used to match by accident through the 1x1 array:
- 320d51765af9b6b7 A376 `FTEST(TRANSPOSE(SUM(Data!D12)),range)` expected `#DIV/0!`, ours `#VALUE!`: fixed in `fn_f_test` (fisher.rs) by taking a computed `CalcResult::Number` (not a one-cell reference, which stays `#VALUE!`, ba4ddb817a8e700e A345) as a sample of one. That alone fixed 284 gen-m9 cells; the last full run (before the reference guard) showed A345 regressing, the guard is built into the tree but its eval was not re-run to completion (build timed out).
- 4e55500bb8ad3cd8 A178 `LINEST(Data!C8:C9,<num>,UNIQUE(<num>,TRANSPOSE("1")))` expected `#DIV/0!`, ours `#VALUE!`: `fn_unique` casts the scalar text "1" by_col with `cast_bool_ref` and gives `#VALUE!`. Unknown whether Excel's UNIQUE accepts "1" or LINEST reports `#DIV/0!` ahead of the UNIQUE error; needs an `excel-probe` (`UNIQUE(1,"1")`, `UNIQUE(1,{"1"})`, `LINEST` with an error x).
Next: re-run `make eval-baseline` (must show REGRESSIONS 0), resolve A178 with a probe, add a unit test in test_fn_transpose.rs, update docs/excel-semantics (text.md line ~1024 "Ours lifts" is now stale; add the F.TEST scalar quirk to statistics.md), `make check`, review.
