---
id: T-3693
title: "Gate A slice 4l: reference table for AGGREGATE's array forms (functions 14-19)"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3551]
---

## Goal
AGGREGATE has a committed, oracle-filled `s1` Gate A table over its array forms, functions 14-19 (LARGE, SMALL,
PERCENTILE.INC, QUARTILE.INC, PERCENTILE.EXC, QUARTILE.EXC), with a cited mpmath definition.

## Context
- Decision 0228 (T-3551): functions 1-13 take references only (numbers.md "AGGREGATE"), so the gen suites and probes
  drive them; functions 14-19 take an array and k, which the generator's one-row array literal (T-3586) can write, and
  they have floating-point kernels: `AGGREGATE(18,0,{3,1,4,1,5,9},0.3)` is 1.2000000000000002 in Excel (probes/aggregate-grid
  H45). So AGGREGATE needs a table, drawn over function_num 14-19.
- The definition: the order statistic (14, 15) or the inclusive/exclusive percentile and quartile interpolation (16-19)
  of the array's doubles; options 0-7 do not change an array-literal argument's numbers (an array literal holds no
  hidden rows or errors when drawn as numbers). Cite statistics.md's percentile entries and numbers.md "AGGREGATE" and
  "AGGREGATE 14-19: k is read before the array" (statistics.md).
- k outside the valid range (k < 1 or above the count for 14 and 15; outside [0, 1] for .INC, (0, 1) and the
  count-dependent bounds for .EXC) is a domain error.

## Acceptance criteria
- [ ] A definition for AGGREGATE over function_num 14-19, with tests against the probes/aggregate-grid H41-H46 values.
- [ ] `eval/accuracy/tables/AGGREGATE/s1.{case,math,oracle}.json` generated, filled (excel-graph) and committed;
      `rustsheet-eval accuracy` reports `missing tables` lower by 1; failing points owned (`known-failures.tsv`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE 1-13 (reference-only; the eval suites drive them).
