---
id: T-2066
title: "Probe QUARTILE of a single text cell with quart 7 and 1 (gen-m9 288b99797f58f149 A3)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
QUARTILE of a single text cell (" x ") with quart 7 and quart 1 has Excel-evidenced results, and the engine matches them.

## Context
gen-m9/288b99797f58f149 Calc!A3 `=BETAINV(B7,QUARTILE(D6,D5),B5)+..` (D6 = " x ", quart 7): Excel `#NUM!`, engine `#VALUE!`. BETAINV's invalid probability may decide it alone, so use the excel-probe skill on `QUARTILE(<cell " x ">,7)` and `QUARTILE(<cell " x ">,1)` directly. Code: functions/statistical/percentile.rs. Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error". Found by T-1403.

gen-m9 sweep (gap triage 2026-10-23, T-2255). This task's cell, now in the gated gen-m9 baseline (the sweep gated the suite).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/288b99797f58f149 Calc!A3 `=(BETAINV(Data!$B$7,QUARTILE(Data!$D6,Data!D5),Data!B5)+MIN(Data!C2))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/288b99797f58f149.case.json (1): Calc!A3

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2255): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The two probe cells (and `PERCENTILE.EXC(<no-number range>,1/0)` = `#DIV/0!`) are in eval/suites/probes with oracle values; gen-m9/288b99797f58f149 A3 passes or its cause is recorded in statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Outcome
A single cell holding text (numeric-looking text too) is now empty data for PERCENTILE*/QUARTILE*/PERCENTRANK* (`#NUM!`,
not `#VALUE!`); probes quartile-single-text-cell (B1-B28) and quartile-numeric-text-cell (C1-C9) carry excel-graph values;
gen-m9/288b99797f58f149 A3 passes. PERCENTRANK reports an error x before its empty-data `#N/A`, and QUARTILE.EXC and
PERCENTILE/PERCENTILE.INC report k's error before the empty-data `#NUM!` (`PERCENTILE(<no number>,1/0)` is `#DIV/0!`, C9).
The XNPV piece these surfaced was split into T-2599 (#1690) and lands first. Eval: see the PR (gen-m9 and probes known
failures down, 0 regressions).

## Handoff (resumed)
Done and pushed on this branch: PERCENTILE/QUARTILE/PERCENTRANK single-text-cell fix, PERCENTRANK error-x-first, QUARTILE.EXC quart-first, probes (oracle values filled), docs, baseline (94 gen-m9 fixed, 0 regressions). `make check` passes except eval-check's 60 s timeout of probes/union-walk-repeated-in-lambda on this box (use `scripts/eval_guard.sh target/release/rustsheet-eval update-baseline --timeout 400`).
Reviews: security APPROVE, quality APPROVE (minors fixed), architecture REQUEST_CHANGES: the XNPV fix (financial lane) must be its own task. Filed as T-2599. Next: land T-2599 first (cherry-pick the xnpv hunk, probe, financial.md entry, matching baseline), then drop those pieces from this branch, merge main, re-run `update-baseline`, `make check`, reviewers, and confirm PERCENTILE.EXC(<no-number range>,1/0) is probed (the third architecture note).
Excel-fidelity review: APPROVE. Unprobed variants to add to the probes (minor, non-blocking): a single cell of numeric-looking text (`QUARTILE(A1,1)`, `PERCENTRANK(A1,5)` with A1 = "5"); for T-2599, `XNPV(1,A2:A2,B11)` (one-cell values range, still `#NUM!` here) and an empty single dates cell `XNPV(1,B11,A2)`; and replace the `error == Error::NUM` values/dates flag in `get_array_of_numbers_xpnv` with an explicit parameter.
