---
id: T-3643
title: "IMLOG10, IMLOG2, IMSQRT and IMSECH result text by region: sweep both parts across the domain and match Excel's text"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMLOG10, IMLOG2, IMSQRT and IMSECH print Excel's text (value and formatting) in every region of the complex plane, with any difference found stated as a rule on the region, not a list of points.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports these four producing text that differs from Excel's in some regions. Done work:
T-2117 (hypot modulus of IMLN/IMLOG10/IMLOG2/IMSQRT), T-2530 (IMLOG2's ln 2 division), T-2539 (logs at 1E+200 and
1E-200+1E-200i), T-2572 (IMSQRT range rule, subnormal-square flush), T-2536 (IMSQRT of a negative real), T-2537 and
T-2743/T-2750 (IMSECH last digits). Open owners: T-2894 (subnormal squares whose sum is normal), T-3609 (IMSECH below
|x| = 0.01). No task has swept the four over one grid covering every quadrant, the axes, |z| near 1 (log near 0),
huge and tiny parts, and parts of very different magnitude. Code: functions/engineering/complex/ (log and sqrt
kernels, `engineering::complex::parse` formatter).
Sweep each function over a grid of (x, y) by sign and magnitude (0, 1E-300, 1E-20, 1E-8, 1-1E-15, 1, 1+1E-15, 10,
1E8, 1E20, 1E300) and both orderings of a large and a small part. Classify misses: wrong value (more than 15 digits
off), wrong format (exponent, i placement, sign of zero), wrong type.

## Acceptance criteria
- [ ] A probe case (excel-graph) gives Excel's text for the grid for all four functions.
- [ ] Every grid cell where the engine is more than a 15-digit unit off, or prints a different shape or type, matches Excel, or is listed in T-2894 or T-3609 when it falls in their bands.
- [ ] Cells within 15 digits are left to a follow-up with `defer: phase-1b`.
- [ ] Every quirk found is written up in docs/excel-semantics/engineering.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- The trig and hyperbolic #NUM! domain (T-3642).
- Bit-level fits.
