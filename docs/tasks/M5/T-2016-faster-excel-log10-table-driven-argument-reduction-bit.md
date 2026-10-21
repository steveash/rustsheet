---
id: T-2016
title: "Faster excel_log10: table-driven argument reduction, bit-identical to the x87 model"
milestone: M5
lane: perf
status: done
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
- [x] A release microbenchmark of `excel_log10` before/after in the Outcome.
- [x] Bit-identical to the current kernel on a large random sweep (a unit test), and probes/log10-kernel stays 368/368.
- [x] probes/log10-kernel gains edge inputs the T-2004 review found missing (1±2^-52, 1±1e-10, 5e-324,
  2.2e-308, 1.7976931348623157e308; an x87 library might take FYL2XP1 near 1), oracle-filled and passing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the Excel model.

## Outcome
- `x87_log10.rs`: `ln_reduced` takes ln f for f in [√2/2, √2] as ln(i/128) from a 91-entry double-double table plus
  2 atanh(s), s = (f - c)/(f + c), |s| < 2^-8.5: 8 atanh terms (4 in double-double, 4 in double) instead of 23, and
  the double-double products by `x87_exp::dd_mul_in_range` (Veltkamp `two_prod`, no FMA call) instead of `dd_mul`.
  LOG10, one-argument LOG, MULTINOMIAL and LN share it.
- Release microbenchmark (1M random x over 1e-20..1e20, 5 passes, this container): `excel_log10` 468 ns/call before,
  198-202 ns after (glibc `log10` 8.6-9.5 ns), about 2.3x. The rest is the double-double division and the 64-bit
  double rounding (`dd_to_extended`, `round_extended_to_double`); an inlined FMA-free division gained nothing measurable.
- Bit-identity: unit test `the_table_returns_the_untabulated_bits` checks LOG10 and LN against the old 23-term kernel
  (kept in the tests) on about 780,000 inputs: 400,000 random doubles (subnormals included), every double within 2^12
  ulp of 1, √2, √2/2, each table point and each cut between points, and the edge inputs. The LN fixture test (4,030
  points) passes unchanged.
- probes/log10-kernel rows 93-98 (excel-graph, 2026-10-21): 1+2^-52, 1-2^-52, 1-2^-53, 1+1e-10, 1-1e-10 and the
  largest double, LOG10 and LOG; the case is 380/380. Excel cannot reach the kernel with a subnormal from a cell: a
  subnormal cell constant reads as 0 (`LOG10` `#NUM!`; probes/subnormal-cell-constants, numbers.md). That case's 12
  failing cells are new coverage for follow-up T-2228; visible-parity lock 1765 -> 1777 for that new case only.
