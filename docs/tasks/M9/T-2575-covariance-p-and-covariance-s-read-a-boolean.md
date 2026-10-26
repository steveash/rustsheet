---
id: T-2575
title: "COVARIANCE.P and COVARIANCE.S read a boolean scalar argument as one value (size #N/A, one point #DIV/0!), not #VALUE! (gen-m9, 4 cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
COVARIANCE.P and COVARIANCE.S take a TRUE or FALSE argument, typed or computed, as one value, so the usual size `#N/A` and one-point `#DIV/0!` apply instead of `#VALUE!`, as Excel does.

## Context
From T-2436's probe of the untraced gen-m9 cells. statistics.md "Array arguments given a scalar, a single cell or an error" (one value per scalar) and "COVARIANCE: a boolean scalar is one value (T-2436)". COVAR is unprobed for the boolean; check it in the same probe. Code: functions/statistical/covariance.rs and paired.rs.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): a TRUE or FALSE argument (typed or computed) is one value: `COVARIANCE.P({1,2,3},TRUE)` is `#N/A` and `COVARIANCE.S(-2.5,FALSE)` `#DIV/0!` (probes/t2436/computed-scalar A35:A37); in the sweep `COVARIANCE.P({1E+15,1,-1E+15},IF(..)<TRUE>)` and `COVARIANCE.P(Data!B3:E7,CHOOSE(1,("abc">Data!A4)))` are `#N/A`, `COVARIANCE.S(YEAR(20863),HYPERLINK(..,("x~*"=Data!C9)))` and `COVARIANCE.S(Data!D10,IF(<blank>,-2.5))` `#DIV/0!` (ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A356 `=COVARIANCE.P({1E+15,1,-1E+15},IF(Data!C9,(3.14159>Data!D11)))`: Excel #N/A, ours #VALUE!
- gen-m9/899f9555e67da2d6 Calc!A369 `=COVARIANCE.P(Data!B3:E7,CHOOSE(1,("abc">Data!A4)))`: Excel #N/A, ours #VALUE!
- gen-m9/f34f66fb7f81e673 Calc!A127 `=COVARIANCE.S(YEAR(20863),HYPERLINK("http://example.com",("x~*"=Data!C9)))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/bf979a17ab5aafcb Calc!A353 `=OCT2HEX(COVARIANCE.S(Data!D10,IF(Data!D6,(-2.5))),10)`: Excel #DIV/0!, ours #VALUE!; inner `COVARIANCE.S(Data!D10,IF(Data!D6,(-2.5)))` Excel #DIV/0!, ours #VALUE!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A356
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A369
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A127
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A353

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.
