---
id: T-2087
title: "Probe IMEXP overflow cutoff at 2^-26 and near y = pi (T-2060)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMEXP's overflow cutoff in the imaginary part is pinned exactly, and the behavior near y = pi is known.

## Context
T-2060 found `IMEXP("1000+yi")` is `#NUM!` for 1.4E-8 and capped for 1.5E-8; `imexp` uses sqrt(DBL_EPSILON) = 1.4901161193847656E-8 as a hypothesis. Probe `1000+1.4901161193847E-8i` and `1000+1.4901161193848E-8i`, and y within 1E-8 of pi (e.g. `1000+3.14159265358979i`, where cos y is near -1); `IMEXP("1000+2E8i")`-style y past 134217727 (libm fallback is a guess), and `DBL_MAX&""` / TEXT of DBL_MAX (T-2060 prints `1.79769313486232E+308` in IM* text); fix `imexp` and engineering.md (IM* results that overflow).

## Acceptance criteria
- [ ] Probe cases recorded with oracle values; `imexp` matches them; engineering.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/eng-imexp-overflow-cutoff, -pi, -pi2, -halfpi; excel-graph). The cutoff around 0 is exactly 2^-26; around other multiples of pi/2 it is a smaller window bracketed to 1.05-1.06E-8 (fitted as 2^-26/sqrt(2)); |y| >= 2^27 is `#NUM!` for any x. `imexp` and engineering.md updated. Eval: 4 new probe cases, all passing, no regressions. Open: the exact window around non-zero multiples and TEXT of a DBL_MAX literal (excel-graph rejects such a workbook).
