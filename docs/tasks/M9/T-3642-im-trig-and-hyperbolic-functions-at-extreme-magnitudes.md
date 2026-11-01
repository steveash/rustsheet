---
id: T-3642
title: "IM* trig and hyperbolic functions at extreme magnitudes: map where Excel gives #NUM! over both parts, all ten functions"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
For IMSIN, IMCOS, IMTAN, IMSEC, IMCSC, IMCOT, IMSINH, IMCOSH, IMSECH and IMCSCH, the engine gives `#NUM!` exactly where Excel does over the whole range of both parts, very large and very small, and the rule is one stated overflow/loss-of-meaning test per function family, not a list of points.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

Excel returns `#NUM!` for some inputs where the engine still returns a complex string, mostly at large or tiny
components. Earlier work pinned edges one function at a time: T-1827/T-2003 (IMSIN, IMCOS, IMSINH, IMCOSH overflow),
T-1966 and T-2760 (IMTAN/IMCOT past cosh overflow, near pi/2), T-2334, T-2335, T-2540 (zero and tiny results), and
the open probes T-2631 (log-space edge of IMSIN/IMCOS 709<|y|<709.78, IMSECH/IMCSCH past |x|=709, IMCOSH), T-3616
(IMSIN/IMCOS cut 709.7-709.75), T-2668 (IMCSCH tiny), T-2914 (IMCSCH of a large real). No task sweeps the whole
family on one grid, and IMTAN, IMSEC, IMCSC and IMCOT have no open overflow owner at all. Code:
crates/rustsheet-engine/src/functions/engineering/complex/ (the trig and hyperbolic kernels and their overflow
checks); engineering.md "IM* results that overflow (T-1827)".
Probe a log-magnitude grid of x and y (each of 0, ±1E-308..±1E-300, ±1E-200, ±1E-20, ±1, ±20, ±700..±720 in steps
fine enough to bracket each cut, ±1E10, ±1E100, ±1E300) for all ten functions, so that each cut found becomes a rule
that holds across functions sharing a kernel. Where a cut falls in a band an open task owns (T-2631, T-3616, T-2668,
T-2914), hand the cells to it rather than duplicating it.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the grid for all ten functions; each function's #NUM! region is stated in engineering.md as a rule on x and y.
- [x] Every grid cell outside the bands T-2631, T-3616, T-2668 and T-2914 own has Excel's type (number text versus `#NUM!`); cells inside them are listed in those tasks.
- [x] Last-digit differences on cells that are text in both are left to existing last-digit owners or a follow-up with `defer: phase-1b`.
- [x] Every quirk found is written up in docs/excel-semantics/engineering.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Last-digit text fits (T-2538, T-3609 and the like).
- IMEXP, IMPOWER, IMLN/IMLOG*, IMSQRT (T-3643).

## Outcome
Probes probes/im-trig-magnitude-grid (4360 cells: a 19x19 log-magnitude grid of both parts from 0 and 1E-308 to
1E+300, 700 to 720 around exp overflow, plus sign variants, all ten functions) and probes/im-trig-argument-cut (190
cells around 2^27), excel-graph, 2026-11-01. The one missing rule was SIN's argument limit on the complex paths:
IMTAN, IMSEC, IMCSC and IMCOT are `#NUM!` from |x| = 2^27 and the hyperbolic four from |y| = 2^27, IMSIN and IMCOS from
|x| > 134217727 (the inner-sine constant IMPOWER and BESSELJ share), except where Excel takes no sine (IMTAN and IMCOT
at `±i`, the reciprocals once 2e^-|other part| is subnormal). complex/trig.rs implements it (`past_trig_limit`,
`inner_trig_overflows`, `reciprocal_flushes`); engineering.md "IM* trig and hyperbolic functions past SIN's argument
limit (T-3642)" states the rules. Before: 475 grid cells off (360 `#NUM!` in Excel, text ours). After: the only type
mismatches left are the 32 IMSIN/IMCOS cells at y = 709.78, listed in T-3616; 41 IMCSCH cells at x = 700 to 709 where Excel keeps normal tiny parts and ours
prints 0 are listed in T-2668 (the real rows also in T-2914); 74 grid and 37 cut cells differ in last digits (libm
sine in the complex IMSIN/IMCOS/IMTAN paths), filed as T-3758.
Eval delta: probes +2 cases, 4213/4360 and 153/190 cells pass, the misses recorded as new-case known failures; every
other suite unchanged, 0 regressions.
