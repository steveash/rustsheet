---
id: T-0987
title: "PRODUCT and SUMPRODUCT flush subnormal results and literals to 0"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT and SUMPRODUCT return 0 for subnormal results and read subnormal literals as 0, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Subnormal results become 0, per function group); evidence probes/subnormal-function-results (T-0189). Engine gives 1e-310 for PRODUCT(1E-200,1E-110) and SUMPRODUCT({1E-200},{1E-110}), 1e-308 for PRODUCT(1E-308,1). Reuse arithmetic::finite_or_num.

## Acceptance criteria
- [ ] probes/subnormal-function-results A9, A10, A41 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
