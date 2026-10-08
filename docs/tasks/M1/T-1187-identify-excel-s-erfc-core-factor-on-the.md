---
id: T-1187
title: "Identify Excel's erfc core factor on the t1040 grid"
milestone: M1
lane: research
status: todo
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
- [ ] A candidate reproduces at least 95% of the t1040-erfc grid points with x >= 1 bit-exactly, and A5. Otherwise the statistics.md entry records the new negatives and the task is closed as not found.
- [ ] If one is found, it is implemented with no regression in ERFs.xlsx, NORM_DIST.xlsx, GAUSS.xlsx and the probes suites, and T-1186 is closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
