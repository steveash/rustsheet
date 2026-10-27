---
id: T-2717
title: "Generator M2 criteria pool: add text-number criteria"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Generated COUNTIF/COUNTIFS/SUMIF/SUMIFS (and AVERAGEIF/AVERAGEIFS, which share the pool) formulas also exercise
criteria that are numbers written as text, so gen suites score Excel's text-number criteria matching.

## Context
- Found by T-2673 (gen-m2 sweep): `CRITERIA_POOL` in `tools/src/rustsheet_tools/gen/generator.py` has wildcards,
  comparison operators, `""`, `"<>"`, `"abc"` and the numeric literal `1`, but no text-number criteria such as `"1"`,
  `"2.5"`, `"=2"` against numeric cells or `"1E1"`.
- The T-2673 duplicate run's Excel-fidelity review (PR #1809, closed as a duplicate of #1806) found three more gaps in
  the same pool: no tilde-escaped wildcard (`"x~*"`, `"~?"`), no date criterion (`">"&DATE(2020,1,1)`, a date as
  text) and no boolean-text criterion (`"TRUE"`, `"=FALSE"`). The data pools need values those criteria can match.
- Existing case files are stored, so changing the pool only changes newly generated cases.

## Acceptance criteria
- [x] `CRITERIA_POOL` gains a few text-number criteria (for example `"1"`, `"2.5"`, `"=2"`, `"1E1"`), a tilde escape, a date
      criterion and a boolean-text criterion; the generator's unit tests cover that they are emitted as quoted text.
- [x] A small gen-m2 batch (`--focus --functions COUNTIF,COUNTIFS,SUMIF,SUMIFS --count 2 --seed 2717`) is generated,
      filled with excel-graph and baselined (0 regressions); any new mismatch family gets a functions task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing criteria matching (the triaged functions task does that).

## Outcome
`CRITERIA_POOL` gained text-number (`"1"`, `"2.5"`, `"=2"`, `"1E1"`), tilde-escape (`"~?"`; `"x~*"` was already a
`TEXT_POOL` data value), boolean-text (`"TRUE"`, `"=FALSE"`) criteria, plus a new `CRITERIA_EXPR_POOL` of date criteria
(`">"&DATE(2020,1,1)`). Unit test covers emission. Batch `--focus --functions COUNTIF,COUNTIFS,SUMIF,SUMIFS --count 2
--seed 2717` filled with excel-graph: 2 cases, 80 cells, all bit-exact (0 mismatches, 0 regressions), so no new
functions task. gen-m2: 39 -> 41 cases, 100%.
