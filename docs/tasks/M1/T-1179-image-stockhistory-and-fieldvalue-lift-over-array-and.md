---
id: T-1179
title: "IMAGE, STOCKHISTORY and FIELDVALUE lift over array and range arguments; IMAGE of non-URL text is VALUE"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMAGE, STOCKHISTORY and FIELDVALUE give Excel's results for array and range arguments and for IMAGE sources that are not a URL.

## Context
probes/rich-errors-arg-kinds (T-1141, excel-graph) fails for these cells; see information.md "IMAGE and STOCKHISTORY never fetch".
- All three functions lift over an array or range argument: `ROWS`/`COLUMNS` of `IMAGE({"a","b"})` are 1 and 2, of `STOCKHISTORY(Q1:Q2,DATE(2020,1,2))` 2 and 1, of `FIELDVALUE(5,{"a","b"})` 1 and 2, of `FIELDVALUE({1},"Price")` 1 and 1, and `FIELDVALUE(Q1:R2,"Price")` spills 2x2. The engine returns one scalar error for every lifted form above (the F entries for rich-errors-arg-kinds in eval/baseline/probes.txt list them; static_analysis.rs signatures say scalar). Only the probed arguments are known to lift: IMAGE `source`, STOCKHISTORY `stock`, FIELDVALUE `value` and `field_name`.
- `IMAGE` of non-URL text (`"a"`, `""`) and of a boolean is `#VALUE!` (ERROR.TYPE 3), the engine gives `#BLOCKED!`. A well-formed URL stays `#BLOCKED!` (deviation, decision 0014).
- `FIELDVALUE(Q1:Q1,"Price")` (a 1x1 range) is a scalar `#FIELD!`.

## Acceptance criteria
- [ ] The failing cells of probes/rich-errors-arg-kinds pass, except any left pending for an undecodable rich error type; `make eval-baseline` records the fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
A reachable URL staying `#BLOCKED!` (decision 0014: no fetch) and the pending undecodable cells.

## Outcome
IMAGE, STOCKHISTORY, FIELDVALUE and ERROR.TYPE lift over array/range arguments; non-URL IMAGE sources are #VALUE!. probes/rich-errors-arg-kinds now passes all 70 cells (was 42).
