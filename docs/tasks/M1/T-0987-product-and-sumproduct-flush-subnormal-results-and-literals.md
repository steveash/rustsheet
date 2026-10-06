---
id: T-0987
title: "PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE flush subnormal results and literals to 0"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE return 0 for subnormal results and read subnormal literals as 0, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Subnormal results become 0, per function group); evidence probes/subnormal-function-results (T-0189). Engine gives 1e-310 for PRODUCT(1E-200,1E-110) and SUMPRODUCT({1E-200},{1E-110}), 1e-308 for PRODUCT(1E-308,1), 1.5e-308 for PRODUCT(3E-308,0.5), 1e-320 for SUMSQ(1E-160), 5e-310 for AVERAGE(3E-308,-2.9E-308). Addition and SUM keep subnormals in Excel: do not flush them. Reuse arithmetic::finite_or_num.

## Acceptance criteria
- [x] probes/subnormal-function-results A9, A10, A41, A63, A64, A66, A67, A68 pass; baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`arithmetic::flush_subnormal` (also used by `finite_or_num`) now flushes the results of PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE; SUM keeps subnormals. Probes known failures 279 -> 271 (fixed 8: A9, A10, A41, A63, A64, A66-A68), no regressions; unit test added.
Subnormal literal arguments are not read as 0 by these functions (only results are flushed); that and the other average functions are filed as T-1007.
