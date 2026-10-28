---
id: T-2531
title: "Probe IMCSC/IMCOT of tiny imaginary parts (1E-7i, 1E-9i, 1E-9+1E-9i) where cosh 2y rounds to 1"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answers for IMCSC and IMCOT of tiny imaginary parts are evidenced, and ours match them.

## Context
T-2044 fitted IMCSC (`t = sin²x + (cosh 2y − 1)/2`) and IMCOT (`t = cosh 2y − (2cos²x − 1)`) on
probes-numeric im-last-digit-sweep, whose smallest argument is 1E-5 (rows 65, 70: Excel loses ~8
digits there, which the fit reproduces). Below |y| ≈ 1.05E-8, `cosh 2y` rounds to 1, so our `t` is
0 and both functions now answer `#NUM!` (`IMCSC("1E-9i")`, `IMCOT("1E-9i")`; before T-2044 about
-1E+9i), and between 1E-8 and 1E-6 only a few digits survive. Unprobed (engineering.md "IM*
reciprocal kernels, IMABS and IMARGUMENT", UNVERIFIED line). Code: `imcsc`, `imcot` in
crates/rustsheet-engine/src/functions/engineering/complex/trig.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with IMCSC and IMCOT of 1E-6i, 1E-7i, 1E-8i, 1.1E-8i, 1E-9i, 1E-9+1E-9i,
      1E-20i and 1+1E-9i (text plus IMREAL/IMAGINARY), with oracle values.
- [ ] engineering.md records the result; the UNVERIFIED line goes; any mismatch is fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe case `probes-numeric/im-csc-cot-tiny-imag` added with excel-graph values; our IMCSC/IMCOT match on all 56 cells, so no code change. Result recorded in engineering.md, UNVERIFIED line removed. Eval delta: +56 passing cells (new case, no baseline change).
