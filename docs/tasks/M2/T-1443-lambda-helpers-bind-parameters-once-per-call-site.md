---
id: T-1443
title: "LAMBDA helpers bind parameters once per call site instead of re-patching the body per element"
milestone: M2
lane: perf
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BYROW, BYCOL, MAP, SCAN, REDUCE and MAKEARRAY call their LAMBDA once per element without cloning the LAMBDA body and re-running `assign_variable_ids` on every call, so a whole-column BYROW (1,048,576 calls) costs one body walk, not one per row.

## Context
Raised by the performance review of T-1395: `call_lambda` / `call_lambda_with_values` (functions/logical/lambda.rs) clone the body and patch variable ids per call, which is O(body) of allocation per element. Patch once per helper call and rebind the variable-stack slots per element. Behavior must not change (existing BYROW/BYCOL/MAP/SCAN/REDUCE/MAKEARRAY tests and eval stay green).

## Acceptance criteria
- [ ] The LAMBDA body is patched once per helper call; a benchmark or test shows `BYROW(C:C,LAMBDA(r,1))` no longer allocates per row for the body.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
