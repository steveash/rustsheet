---
id: T-1012
title: "Probe IMAGE/STOCKHISTORY/FIELDVALUE argument errors and IS* of rich errors (SPEC-0003 R1)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Oracle evidence for the behavior SPEC-0003 marks "needs evidence", so T-1013 implements what Excel does.

## Context
SPEC-0003 "Excel behavior to match" (rollout R1). Add `eval/suites/probes/cases/rich-errors-args.case.json` (excel-probe skill) and fill it with excel-graph. Every call must use a valid arity: an arity-invalid call makes Excel for the web reject the whole workbook (T-0040). Do not write `=#BLOCKED!`-style literals either (syntax.md).
Questions:
- `IMAGE(1/0)`, `IMAGE("x",1/0)`, `IMAGE(A1)` with A1 empty or a number;
- `_xlfn.STOCKHISTORY(1/0,DATE(2020,1,2))` and `("QZXQZXQ",1/0)`;
- `_xlfn.FIELDVALUE(1/0,"Price")`, `(A10,1/0)` and `({1,2},"Price")` (scalar or array?);
- for an IMAGE error cell: ISERR, ISNA, TYPE, `IFNA(...,"y")`, `ERROR.TYPE(...+#DIV/0!)`, and `COUNTIF(<range holding it>,"#BLOCKED!")`;
- the same IS* questions for STOCKHISTORY and FIELDVALUE error cells.

## Acceptance criteria
- [ ] The case and its `.oracle.json` sidecar are committed, with no pending cells.
- [ ] Each finding is recorded in docs/excel-semantics (information.md and arrays.md) with cell evidence, and SPEC-0003's "needs evidence" points are updated to cite it.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions (T-1013).
