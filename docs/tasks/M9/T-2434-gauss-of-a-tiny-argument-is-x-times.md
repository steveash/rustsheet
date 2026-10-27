---
id: T-2434
title: "GAUSS of a tiny argument is x times phi(0), not 0 (gen-m9, 8 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: accuracy
---

## Goal
GAUSS(z) for a tiny z returns Excel's z times phi(0) (`GAUSS(1E-300)` = 3.989422804014327E-301, `GAUSS(1E-15)` =
3.989422804014328E-16) instead of 0 or the cancelled NORM.S.DIST(z) - 0.5.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "GAUSS of a tiny argument is x times phi(0) (T-2255)" records it: 8
cells (GAUSS(1E-300) five times, GAUSS(-1E-300) twice, GAUSS(1E-15) once through a cell). Ours computes Phi(z) - 0.5, which
cancels. 3.989422804014327E-301 is the double nearest 1E-300 * 0.3989422804014327 (phi(0) = 1/sqrt(2*pi)); compute
GAUSS from erf(z/sqrt(2))/2 or from z*phi(0) below a cut, and check that every passing GAUSS cell in eval/ still passes
(probes-numeric and gen-m9 have about 250 more).
Code: `fn_gauss` in crates/rustsheet-engine/src/functions/statistical/gauss.rs.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A233 `=GAUSS((-1E-300))`: Excel -3.989422804014327e-301, ours 0.0
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A233
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A225, A347
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A363
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A377
- gen-m9/a54c738c697617c3.case.json (1): Calc!A235
- gen-m9/b4173fe25109a238.case.json (1): Calc!A354
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A34

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): 7 of these 8 known-failure lines disappear (the last, 1 ulp low, is T-2931) from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] No GAUSS cell that passes today fails (state the count checked in the PR).
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAUSS's last digits elsewhere (T-0968).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Outcome
GAUSS returns z * phi(0) for |z| < 2E-15 (Excel still shows the cancelled Phi(z) - 0.5 from about 4E-15 up, so a 1E-8 cut regressed 8 gen-m9 cells). gen-m9 known failures 3724 -> 3717 (7 fixed, 0 regressions, no other suite changed). The GAUSS cells in the other suites (probes, probes-numeric, ironcalc-calc) show 0 regressions in the same baseline run. `GAUSS(1E-15)` (a54c738c697617c3 A235) stays 1 ulp low: T-2931.
