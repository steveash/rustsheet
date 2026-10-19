---
id: T-1914
title: "SUBTOTAL over TAKE of SWITCH/IFS with an array selector spills a 2x2 block of #VALUE! (fuzz: debug_assert panic)"
milestone: M2
lane: engine
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2)) and the IFS twin spill a 2x2 block of #VALUE! as Excel's do.

## Context
probes/subtotal-take-array-selector A30:B31 and A40:B41 (Excel a 2x2 block of #VALUE!, ours a single #VALUE! in A30/A40); the IF twin A20:B20 (1x2 #VALUE!) matches. Found by T-1909; is_read_reference in static_analysis/aggregate.rs (T-1882).

Nightly fuzz run 39196675245 (2026-10-19, main at c31b96ef), failed on
`formula_eval` with this exact input: `SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2))`
(base64 `U1VCVE9UQUwoOSxUQUtFKFNXSVRDSCh7MSwyfSwxLEgxOkgyLDIsSTE6STIpLDIpKQ==`). It is a deadly
signal: the debug assertion "Larger-than-1x1 array reached scalar-context cell" at model.rs:928,
the T-1874/T-1882 panic site, reached because static analysis takes the SWITCH for cells (a
scalar) while it lifts to a 2x2. The steward reproduced it on 30bee011 with a debug engine unit
test for both the SWITCH form and `SUBTOTAL(9,TAKE(IFS({TRUE,FALSE},H1:H2,TRUE,I1:I2),2))`, so
this task is now P0 (decision 0035). Mind T-1882's cost note: never analyse the same subtree
twice (static analysis must stay linear in nesting depth).

## Acceptance criteria
- [ ] Unit tests (decision 0035) of the SWITCH and IFS formulas above pass in a debug build.
- [ ] The listed probes/subtotal-take-array-selector cells pass; the baseline shrinks.
- [ ] Static analysis stays linear in nesting depth (a deep nested SUBTOTAL/TAKE/SWITCH test finishes at once).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
