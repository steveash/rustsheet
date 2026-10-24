---
id: T-2530
title: "IMLOG2 last digit: fit the ln 2 division (COMPLEXs AH19, sweep N18/N19, overflow-edge A104/A107/A108/A109)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMLOG2's real part prints Excel's 15 digits: ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!AH19
(`IMLOG2` of 3.2E-19, Excel -61.4385618977473) leaves eval/baseline/ironcalc-calc.txt, and the
probes-numeric im-last-digit-sweep N column matches in every row.

## Context
T-2044 time-boxed IMLOG2 (docs/excel-semantics/engineering.md "IM* reciprocal kernels, IMABS and
IMARGUMENT", last bullet). Excel's real part is inflated over the exact `log2 |z|` for some inputs
(N18 `IMLOG2("1E-20")` -66.4385618977473, exact -66.43856189774725; probes/eng-im-overflow-edges
A104 `IMLOG2("710.4+0i")` needs at least 1.18 ulps above the exact value) but not others (A108
`IMLOG2("712+3i")` 9.4757462372528 needs at most 0.36 ulps above). Dividing `ln |z|` by the double
one ulp below ln 2 fits 75/77 sweep cells (misses O40, N48) but breaks A108, so no constant factor
fits; next candidates: a different modulus for the log (`0.5·ln(x²+y²)` in extended precision, the
x87 FYL2X of `x²+y²` with y = 0.5/ln 2), or a log of IMLN's 15-digit text. The overflow-edge cells
A104, A107, A109 are also listed by T-2231 (same kernel; whichever lands first takes them).
Code: `imlog2` in crates/rustsheet-engine/src/functions/engineering/complex/mod.rs.

## Acceptance criteria
- [ ] COMPLEXs.xlsx Sheet1!AH19 leaves eval/baseline/ironcalc-calc.txt (`make eval-baseline`), with
      no new known-failure lines in any baseline.
- [ ] The candidate's score over the sweep's N/O columns and eng-im-overflow-edges A104-A109 is
      recorded in engineering.md (>= 99.9% of cells, or time-boxed per decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMLOG10 and IMLN (their own cells under T-2231).
