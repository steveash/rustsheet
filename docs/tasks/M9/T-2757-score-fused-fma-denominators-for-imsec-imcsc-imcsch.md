---
id: T-2757
title: "Score fused (fma) denominators for IMSEC, IMCSC, IMCSCH and IMCOT on the im-last-digit sweep"
milestone: M9
lane: functions-engineering
status: done
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
- [x] Each kernel's fused variants are scored against the oracle cells and the scores recorded in engineering.md.
- [x] No kernel changes unless a fused form fits at least as many cells; no new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- New oracle probes (file a task if the cells do not discriminate).

## Outcome
Scored with a temporary `mul_add` switch (not committed) on every oracle cell of the four functions (gen-m9,
ironcalc-calc, probes, probes-numeric `im-*`). Fused IMCSC misses sweep D33 and fused IMCOT misses P31 and P70, so
both stay as they are. Fused IMSEC and both fused IMCSCH orders tie the current kernels because no cell tells
them apart, so no kernel changes. Recorded in engineering.md "IM* reciprocal kernels"; the discriminating probe is
T-2817 (candidate inputs listed there). Eval delta: none (docs only).
