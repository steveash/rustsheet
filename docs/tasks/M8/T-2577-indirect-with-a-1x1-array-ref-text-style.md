---
id: T-2577
title: "INDIRECT with a 1x1 array a1 argument is #VALUE!, and F.TEST or COVARIANCE.P over that INDIRECT's #VALUE! is #NUM! (gen-m9, 2 cells)"
milestone: M8
lane: functions-lookup
status: in-progress
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
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.
