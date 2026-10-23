---
id: T-2454
title: "OFFSET lifts over array height and width arguments (2x2 reference array)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
OFFSET lifts over array height and width arguments (a 2-element height gives a 2x2 reference array), as Excel does.

## Context
Evidence: probes/t2414-gen-m7-unexplained A141, A151, A171 and docs/excel-semantics/text.md "TEXTAFTER's result takes the shape of an array if_not_found". `ROWS`/`COLUMNS` of `OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2)` are 2 and 2; `INDEX(..,1,1)` is 0. Ours is 1x1. Row and column offset arguments' lifting is unprobed (probe if the fix needs it).

Owned gen-m7 cells (T-2414):
- 4dac39f89709e2fc A2870 `=TEXTAFTER(CHOOSE(FLOOR.PRECISE(Data!C4,0.1),..),"",1,-INT(2^53),0,OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2))`: Excel #SPILL!, ours #VALUE!

## Acceptance criteria
- [ ] probes/t2414-gen-m7-unexplained cells A141, A151, A171 pass; lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7/4dac39f89709e2fc A2870 (Excel #SPILL!, ours #VALUE!) passes and its line leaves eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.
