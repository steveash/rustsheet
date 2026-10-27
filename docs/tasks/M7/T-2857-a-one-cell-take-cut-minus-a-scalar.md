---
id: T-2857
title: "A one-cell TAKE cut minus a scalar error is a scalar error, not a 1x1 array (probes/t2475 P!A279)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An arithmetic operator on a one-cell TAKE (or DROP) cut of a range gives a scalar, as Excel does, so
`TAKE(range,1,1)-(1/0)` is a scalar `#DIV/0!` (TYPE 16), not a 1x1 array holding it.

## Context
T-2622 made TAKE and DROP of a one-cell reference hand on the reference (TYPE 1; docs/excel-semantics/information.md,
"TYPE of TAKE or DROP of a one-cell reference"). An operator over such a cut still yields a 1x1 array in ours.
- probes/t2475-stat-subexpressions P!A279 `(TAKE(D20!C8:E12,D20!B12,D20!$A7)-(D20!A$3/D20!A2))`: Excel `#DIV/0!`
  (`make eval-show`: expected "#2", TYPE 16), ours "64|1x1|#2". It is the sub-expression probe of gen-m9/775d053de498b8c7
  Calc!A235, which passes on its own (its value is the same error).
- probes/t2622-contents-xmatch-take-lambda A44, A46 and A48 (`TAKE(Data!C2:E4,1,1)-(1/0)`) check only the value, not TYPE.

The likely fix: the binary operator reads a one-cell reference result of TAKE/DROP as a scalar, the same way it reads a
plain one-cell reference.

## Acceptance criteria
- [ ] probes/t2475-stat-subexpressions P!A279 leaves eval/baseline/probes.txt, with no new known-failure line.
- [ ] information.md's TAKE/DROP entry gains the operator rule with P!A279 as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DROP evidence (unprobed; T-2622 notes it).
