---
id: T-2757
title: "Score fused (fma) denominators for IMSEC, IMCSC, IMCSCH and IMCOT on the im-last-digit sweep"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know whether Excel's other IM* reciprocal kernels also fuse a product into their denominator sum, as IMSECH does
(T-2750), and adopt a fused form only where it fits at least as many oracle cells as the current kernel.

## Context
T-2750 found IMSECH's `t = cos²y + sinh²x` matches Excel on all 62 finite sweep cells only as
`fma(sinh x, sinh x, cos²y)`. docs/excel-semantics/engineering.md "IM* reciprocal kernels" notes the guess that Excel's
build contracts `a·a + b·b` to an fma; it is unprobed for the other kernels (reviewer note on T-2750). The current
IMSEC, IMCSC and IMCOT kernels already fit every sweep cell, so this mostly confirms or refutes the hypothesis; IMCSCH
(`sinh²x cos²y + cosh²x sin²y`) has no sweep column and needs its gated cells (ironcalc-calc COMPLEXs.xlsx, gen-m9).
Code: `imsec`, `imcsc`, `imcsch`, `imcot` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] Each kernel's fused variants are scored against the oracle cells and the scores recorded in engineering.md.
- [ ] No kernel changes unless a fused form fits at least as many cells; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New oracle probes (file a task if the cells do not discriminate).
