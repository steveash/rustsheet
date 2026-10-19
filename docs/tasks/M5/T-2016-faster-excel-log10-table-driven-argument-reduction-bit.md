---
id: T-2016
title: "Faster excel_log10: table-driven argument reduction, bit-identical to the x87 model"
milestone: M5
lane: perf
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`excel_log10` (x87_log10.rs) costs a few times less per call and returns the same bits.

## Context
T-2004 made LOG10, one-argument LOG and MULTINOMIAL use Excel's x87 `FLDLG2; FYL2X` model: ln x in double-double by a
23-term atanh series over |s| <= 0.1716, then two roundings. The performance review estimated a few hundred ns per call
against about 20 ns for glibc, which matters for `SUMPRODUCT(LOG10(A1:A1000000))`. Remedy it suggested: table-driven
reduction (64-128 double-double entries ln(c_i), f/c_i near 1, |s| about 2^-8), so 6-8 terms give the same accuracy.
Possibly a Ziv-style fast path (plain double estimate, fall back only near a 64-bit rounding boundary), as T-1637 plans
for trig.

## Acceptance criteria
- [ ] A release microbenchmark of `excel_log10` before/after in the Outcome.
- [ ] Bit-identical to the current kernel on a large random sweep (a unit test), and probes/log10-kernel stays 368/368.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the Excel model.
