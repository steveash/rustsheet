---
id: T-1957
title: "INDIRECT and TRIMRANGE lift into an array of references"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0014-reference-array-results.md
depends_on: [T-1956]
---

## Goal
INDIRECT's ref_text and a1 and TRIMRANGE's trim_rows and trim_cols lift into arrays of references on the carrier T-1956 built for OFFSET.

## Context
Design: docs/specs/SPEC-0014-reference-array-results.md (Rollout step 2: INDIRECT and TRIMRANGE as producers on T-1956's `RefGrid`). Split from T-1810. Evidence: docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19). Failing cells: probes/lookup-argument-modes F14:H15 (INDIRECT) and F34:H35 (TRIMRANGE); probes/reference-lift-elements F6:H8, F12, K9, K16 (INDIRECT) and F9:H10, F11, G11, K6 (TRIMRANGE), with their spill members; gen-m9/675763979c1defb6 Calc!A25 `=UNICODE(INDIRECT("Data!A2:A4",VSTACK(Data!C6:E7)))` (Excel `#SPILL!`, engine `#N/IMPL!`) and gen-m9/adb3fd63c7368166 Calc!A34 `=INDIRECT(OFFSET(Data!A3:A5,3,1,3,3),(TANH(1)=Data!B12))` (Excel `#SPILL!`, engine `#VALUE!`). Reuse T-1956's carrier; do not add a second one.

## Acceptance criteria
- [x] The INDIRECT and TRIMRANGE cells above pass; eval/baseline updated with no new known failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- OFFSET (T-1956).

## Outcome
INDIRECT (ref_text, a1) and TRIMRANGE (trim_rows, trim_cols) lift into `RefGrid` via `Model::lift_references`, sharing `ref_lift.rs` with OFFSET; ISREF is FALSE for a lifted call and static analysis spills the broadcast shape. Eval delta: probes known failures 1767 -> 1735 (32 cells fixed, 0 regressions). Unit tests in test_fn_indirect.rs and test_fn_trimrange.rs.
