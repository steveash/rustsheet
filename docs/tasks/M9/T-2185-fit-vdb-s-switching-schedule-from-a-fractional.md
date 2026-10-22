---
id: T-2185
title: "Fit VDB's switching schedule from a fractional start period"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
VDB with no_switch FALSE and a fractional `start_period` returns Excel's doubles.

## Context
T-1975 fitted VDB's charge order for every no_switch TRUE call and for switching calls from a whole start period
(financial.md "VDB charge order"). Switching from a fractional start still uses the old running-total model
(`vdb_total_depr` in functions/financial/financial_depreciation.rs) and misses probes/fin-vdb-schedule A42-A46.

Lead from the T-1975 lab: a restart model passes 89 of the 92 numeric VDB cells. It takes the depreciation through
`start` (the whole-start schedule, prorating the partial period) as D0, then runs the same schedule from cost `cost - D0`
with straight-line denominator `life - start - p` over the window `[0, end - start]`. It fixes A42, A45, A46 and
A48-A49 but misses A43 (one ulp), A44 (factor 1.5, 244.0757706382978 vs 242.0635...) and fin-vdb-modes A9
(`VDB(2400,300,10,9.5,10)` is 0; the old model passes it with its cap rule). LibreOffice's half-life shift
(start >= life/2 with factor > 1) makes it worse (87/92). More probes around A44 and A9 (factor 1.5, start past the
switch, end at life) should separate the hypotheses.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 3 cells, 3 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on VDB 3. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 3 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 3 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/fin-vdb-schedule A42-A46 pass, with fin-vdb-modes still 20/20.
- [ ] `make check` green; reviewers approve.

Replacing that path should delete `vdb_total_depr`, whose `straight_line` parameter is now always true (T-1975 review).

## Out of scope
- The no_switch TRUE path and whole start periods (T-1975).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (VDB from a fractional start period differs from the 3rd digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).
