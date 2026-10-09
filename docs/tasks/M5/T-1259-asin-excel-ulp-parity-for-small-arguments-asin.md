---
id: T-1259
title: "ASIN: Excel ulp parity for small arguments (ASIN(-0.0001))"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ASIN matches Excel bit-for-bit for small arguments such as -0.0001 without regressing the cached ASIN cells.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09): gen-m6/2f6b96385ed3facd Calc!A38 `=ASIN((-0.0001))`: Excel
-0.00010000000016666668, engine (libm `asin`) -0.00010000000016666667 (NumericUlp). In Python, `atan(x/sqrt(1-x*x))` and
`atan2(x, sqrt(1-x*x))` both give Excel's value for x = -0.0001, libm `asin` does not. `fn_asin` is
`single_number_fn!(fn_asin, |f| Ok(f64::asin(f)))` (functions/math_and_trigonometry/mathematical.rs). ASIN currently passes
its cached cells (ironcalc-calc TRIGONOMETRIC.xlsx ASIN_ACOS_ATAN, poi; see T-0939's note), so a new formulation must keep
all of them: fit candidates against every ASIN cell in eval/ plus this one. ACOS's ulp work is T-0939 (a shared
`atan2`-based formulation may serve both).
**Needs an oracle sweep if no candidate fits all cached cells:** do not fit blind; then file a research-lane sweep task
(like T-1005/T-1008) for ASIN over small, mid and near-1 arguments and depend on it.

## Acceptance criteria
- [ ] gen-m6/2f6b96385ed3facd A38 passes, no ASIN cell in eval/baseline/ gains a failure and no new known failures appear.
- [ ] The formulation and its evidence are recorded in docs/excel-semantics/numbers.md (including negative results).
- [ ] `make check` green; reviewers approve.

## Out of scope
- ACOS (T-0939), hyperbolic inverses (T-0917).
