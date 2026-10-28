---
id: T-2577
title: "INDIRECT with a 1x1 array a1 argument is #VALUE!, and F.TEST or COVARIANCE.P over that INDIRECT's #VALUE! is #NUM! (gen-m9, 2 cells)"
milestone: M8
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2576]
campaign: semantics
---

## Goal
INDIRECT with a 1x1 array a1 argument is `#VALUE!` as in Excel, and F.TEST and COVARIANCE.P over that INDIRECT's `#VALUE!` give Excel's `#NUM!`.

## Context
From T-2436's probe of the untraced gen-m9 cells. references.md "INDIRECT: a 1x1 array a1 argument is #VALUE! (T-2436)"; this answers part of T-2347's question for INDIRECT. The F.TEST/COVARIANCE.P `#NUM!` over the result is UNVERIFIED in mechanism (it may be T-2576's marked value); probe it first. Code: functions/lookup indirect (a1 argument reader).

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): INDIRECT with a 1x1 array a1 argument is `#VALUE!` (`INDIRECT("B1",{1})`, `INDIRECT("B1",{TRUE})`; `INDIRECT("B1",TRUE)` is 5 and `{1;1}` lifts to two rows; probes/t2436/computed-scalar A39:A42; ours 5), and F.TEST or COVARIANCE.P over the result is `#NUM!`, not that `#VALUE!`: `F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})` and `COVARIANCE.P(INDIRECT("Data!A1",FILTER(..)<6 numbers>),Data!B11:D12)` (whose INDIRECT is six `#VALUE!` elements) are `#NUM!` (ours `#DIV/0!`, `#VALUE!`). Why the `#NUM!` is UNVERIFIED: probe it first (an INDIRECT-made error element, compared with `{#VALUE!}` and `1/0`-made errors).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/06b4e8f5de409d5e Calc!A261 `=F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})`: Excel #NUM!, ours #DIV/0!; inner `INDIRECT("Data!B3",TOCOL(2,0,Data!A5))` Excel #VALUE!, ours 3.14159
- gen-m9/f2bd5f380ec002bf Calc!A383 `=COVARIANCE.P(INDIRECT("Data!A1",FILTER(Data!B1:B12,(Data!D1:D12>3.14159),Data!A1:A1)),Data!B11:D12)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A261
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A383

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
- Probe probes/t2577-indirect-ref-array (excel-graph, 2026-10-26, 57 questions): the `#NUM!` is the array of
  references, not its `#VALUE!` elements. An OFFSET or INDIRECT lifted into an array of references, passed as a sample,
  is `#NUM!` in F.TEST (either sample), and as the first argument of COVARIANCE, CORREL, PEARSON, SLOPE, INTERCEPT,
  FORECAST, T.TEST, SUMXMY2 and CHITEST. As COVARIANCE.P's second argument it is cells with no number. Findings in
  statistics.md "An array of references is #NUM! in the paired functions" and references.md "INDIRECT: a 1x1 array a1
  argument is #VALUE!" (ref_text too).
- INDIRECT now lifts over a 1x1 array (`lifts_one_by_one`, as OFFSET, TAKE and DROP do).
- `Model::evaluate_sample_arg` (functions/statistical/paired.rs) marks a direct OFFSET or INDIRECT that gives an array
  of references with a reference in it. F.TEST, `fn_get_two_matrices` and `paired_samples` read their samples through it.
- INDEX with one index into a one-element array of references picks that element (A100).
- Eval delta (`rustsheet-eval check`, 0 regressions): gen-m9 fixed 2 (both owned cells); probes fixed 9 (t2436).
  The new probe case holds 9 known failures (3 questions): LET (A64) and SUMXMY2 (A148) are in T-2766, and
  `STDEV(INDIRECT("B1",{1;1}))` (A157, Excel 0) is in T-2065.
