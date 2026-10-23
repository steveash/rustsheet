---
id: T-2454
title: "OFFSET with a one-cell DROP result as height gives one 2x2 reference, not a lifted 1x1"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
`OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2)` is one 2x2 reference as in Excel (ours: a lifted 1x1 of 2x2 references that spills at INDEX).

## Context
Evidence: probes/t2414-gen-m7-unexplained A141, A151, A161, A171 and docs/excel-semantics/text.md "TEXTAFTER's result takes the shape of an array
if_not_found". `ROWS`/`COLUMNS` are 2, 2 and `INDEX(..,1,1)` is one 0 in Excel. `DROP(A3:A5,-2,0)` is the one cell A3 (1E-15), so the mechanism
(why the height gives 2 rows) is UNVERIFIED: probe first `OFFSET(B9:C12,5,1,1E-15,2)`, `OFFSET(..,A3,2)`, `OFFSET(..,{1E-15},2)`, the DROP form with
A3 = 1 or 3, and COLUMNS/INDEX(..,2,2), record in lookup.md, then fix. OFFSET already lifts over true arrays (lookup.md, T-1956).

Owned gen-m7 cells (T-2414):
- 4dac39f89709e2fc A2870 `=TEXTAFTER(CHOOSE(FLOOR.PRECISE(Data!C4,0.1),..),"",1,-INT(2^53),0,OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2))`: Excel #SPILL!, ours #VALUE!

## Acceptance criteria
- [ ] probes/t2414-gen-m7-unexplained cells A141, A151, A161 (+A162, B161, B162), A171 pass; lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7/4dac39f89709e2fc A2870 (Excel #SPILL!, ours #VALUE!) passes and its line leaves eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.
