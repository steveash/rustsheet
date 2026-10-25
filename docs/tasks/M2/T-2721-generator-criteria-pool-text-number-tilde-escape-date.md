---
id: T-2721
title: "Generator criteria pool: text-number, tilde-escape, date and boolean-text criteria"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Generated COUNTIF/COUNTIFS/SUMIF/SUMIFS/AVERAGEIF(S) formulas exercise quoted text-number criteria, tilde-escaped wildcards, date criteria and boolean-text criteria, and those cells are Excel-backed.

## Context
T-2673 Excel-fidelity review (2026-10-25): every criterion in the 9 new gen-m2 cases came from `CRITERIA_POOL`
(tools/src/rustsheet_tools/gen/generator.py:70): `">0", "<5", ">=2", "<>1", "=a", "a*", "*c", "?b?", 1, "abc", "", "<>"`.
It has no quoted text-number criterion (`"1"`, `"1.5"`, `"=1.5"`), no tilde escape (`"x~*"`, `"~?"`), no date criterion
(`">"&DATE(2020,1,1)` or a date serial as text), and no boolean-text criterion (`"TRUE"`, `"=FALSE"`). These are where the
criteria grammar coerces (docs/excel-semantics/ criteria entries; T-0206 criteria-grammar oracle). Its data pools need values
those criteria can match (text numbers, `x*` literal, dates, booleans): check the generator's M2 data pool too.

## Acceptance criteria
- [ ] `CRITERIA_POOL` (or a pool beside it) gains these criteria, and the data pool holds values they match, with a tools unit test.
- [ ] A new seeded gen-m2 batch (at least 2 cases x 400, `--focus --functions COUNTIF,COUNTIFS,SUMIF,SUMIFS`) is filled
      with excel-graph; `make eval-baseline` shows 0 regressions, and any new known failure gets a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing criteria mismatches the batch finds (their own tasks).
