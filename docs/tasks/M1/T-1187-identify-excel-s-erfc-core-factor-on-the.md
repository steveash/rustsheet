---
id: T-1187
title: "Identify Excel's erfc core factor on the t1040 grid"
milestone: M1
lane: research
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The engine's ERFC/ERFC.PRECISE reproduce Excel's last-digit results for x >= 1, including probes/subnormal-function-boundaries A5 (`ERFC(26.4)`), by implementing the algorithm Excel actually uses.

## Context
T-1186 (blocked) established that Excel scales the tail by `exp(-fl(x*x))` (now in `erfc_cf`, crates/rustsheet-engine/src/functions/statistical/special.rs). The core factor erfc(x)·exp(x²) is up to ±5 ulp from correctly rounded on probes-numeric/t1040-erfc. The candidates tried so far match at most 22% of the 206 grid points with x >= 1: bottom-up CF in 7 orders, Cody CALERF, and the asymptotic series. The tried list is in docs/excel-semantics/statistics.md (ERFC sweep). Score every candidate on the whole grid, never on A5 alone. Excel's NORM.S.DIST lower-tail error clusters at 2^-43, which may share the core.

## Acceptance criteria
- [x] (fallback met: not found) A candidate reproduces at least 95% of the t1040-erfc grid points with x >= 1 bit-exactly, and A5. Otherwise the statistics.md entry records the new negatives and the task is closed as not found.
- [ ] If one is found, it is implemented with no regression in ERFs.xlsx, NORM_DIST.xlsx, GAUSS.xlsx and the probes suites, and T-1186 is closed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Closed as **not found**, per the first acceptance criterion's fallback. The grid has 206 points with x >= 1 in
the task text; the oracle sidecar has 209, of which 205 are non-zero, and every candidate was scored on those 205.
docs/research/t1187-erfc-candidates.py (now the fit lab's `erfc-tail` fit, T-1523) re-implements each candidate in IEEE double:
- bottom-up CF in 5 forms, depths 5 to 1000, with 14 prefactor orders
- Lentz top-down CF, 2 forms
- NR incomplete gamma, gcf/gser, with 3 gln variants
The best is 58/205 (28%), and every configuration together covers 174/205. Excel's implied core error is
-3.6..+3.1 ulp and drifts slowly with x. The negatives and the untried families (fdlibm, Chebyshev, MSVC `exp`)
are in docs/excel-semantics/statistics.md, "ERFC core factor: candidates that do not reproduce Excel". The
second criterion does not apply (nothing implemented). T-1186 stays blocked. No follow-up task: fdlibm and a
Chebyshev fit contradict the A5 evidence, and MSVC `exp` cannot be run from these containers. Eval delta: none
(docs only).
