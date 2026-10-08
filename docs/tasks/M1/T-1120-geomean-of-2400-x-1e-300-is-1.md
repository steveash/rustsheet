---
id: T-1120
title: "GEOMEAN of 2400 x 1E-300 is 1.8066704015823645E-300: model Excel's exponent sum"
milestone: M1
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GEOMEAN of 2400 copies of 1E-300 gives Excel's 1.8066704015823645E-300, and the rule behind it is in numbers.md.

## Context
Found in T-1112 (probes/geomean-overflow-cut-3 C7, excel-graph): GEOMEAN of 2400 copies of 1E-300 is
1.8066704015823645E-300, which is 1E-300 * 2^(2048/2400). So Excel's power-of-two sum is 2048 too high
there, while 1100 copies of 1E-300 give exactly 1E-300 (probes/geomean-overflow-cut row 24). The
significand product, 1.339^2400 = 2^1011, is still finite. Probe counts between 1100 and 2400 and other
small magnitudes (5E-205, 1E-200, 1E-100) to find where the 2048 appears. A wrap or clamp of an exponent
counter (an underflow rescale step of 2^2048?) is a guess to test. Engine: `fn_geomean`
(crates/rustsheet-engine/src/functions/statistical/geomean.rs); numbers.md "GEOMEAN is #NUM! when the
product of significands overflows".

## Acceptance criteria
- [x] A probe that brackets where the extra power of two appears, filled from excel-graph; the rule goes in numbers.md.
- [x] The engine matches probes/geomean-overflow-cut-3 C7 and the new cells, with no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Progress (2026-10-08, PR for probes only)
Two bracketing probes landed, probes/geomean-exponent-sum and -sum-2 (62 cells, excel-graph). Their
findings are in numbers.md "GEOMEAN is #NUM! when the product of significands overflows". The excess is
always 2048 * k, and k steps with n at counts that depend on the value. Powers of two never get it. There
is also an unexplained positive-exponent `#NUM!` (2^1000 x 1500, significand 1). Models ruled out: a threshold on the exponent
sum alone, on the count alone, and on the [1, 2) significand product alone. Untested ideas: a [0.5, 1)
frexp significand product with an underflow rescale step that is miscounted by 2048; a powers-of-two exact
path; a running plain product rescaled by 2^±1024 steps. The engine still misses these 36 new cells and
geomean-overflow-cut-3 C7 (new-coverage failures, baselined).

## Outcome
Fitted the 2048 * k excess: k counts the wraps, modulo 2048, of the exponent of the [0.5, 1) significand
product `P * 2^-n` below the double range, k = ceil((n - floor(log2 P) - 1022) / 2048). It predicts k for
all 41 probed cells with P above 1. Powers of two (P = 1) never gain. `fn_geomean` adds 2048 * k to the
power-of-two sum on the out-of-range path; the rule is in numbers.md. Eval: probes known failures
188 -> 163 (25 fixed, including geomean-overflow-cut-3 C7, no regressions). Still failing: 4 one-ulp root
roundings (geomean-exponent-sum C2, C7; -sum-2 C24, C28; T-1108), 2^1000 x 1000 two doubles low (C11,
T-1108), and the 2^1000 x 1500+ `#NUM!` (C12-C18), which is not the wrap: follow-up T-1156.
