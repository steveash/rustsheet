---
id: T-1542
title: "Student-t quantile at tiny p: match Excel for df >= 99999 and to the last digit for small df"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
T.INV, T.INV.2T, TINV and CONFIDENCE.T at a tiny p match Excel to the last digit for every df, including df >= 99999 at p <= 1E-200 (T.INV(1E-300,99999) = -79.06351062982456).

## Context
T-1531 stopped the `statrs` panic with a tail asymptotic (`t_tail_quantile` in `functions/statistical/t_dist.rs`) but only 7 of 105 cells of probes-numeric `t1531-t-tiny-p` are bit-exact: most are about 1E-13 relative off, and df >= 99999 at p <= 1E-200 is wrong (37.17, the normal quantile, where Excel gives 79.06, because for df = 99999 the `statrs` cdf saturates in the bisection branch and for df = 1E5 the df >= 1E5 path uses a normal series; both paths need a fix). See statistics.md (Student-t quantile at a tiny probability). T.INV.2T with p in (1,2) is also `#NUM!` for us where Excel gives a negative number (cells A121-A128).

## Acceptance criteria
- [ ] All 105 cells of `t1531-t-tiny-p` pass or the residual is a recorded DEVIATION.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
