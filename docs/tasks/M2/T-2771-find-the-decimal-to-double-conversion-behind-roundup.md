---
id: T-2771
title: "Find the decimal-to-double conversion behind ROUNDUP's subnormal cut (probes/t2756 A52-A58)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ROUNDUP, ROUNDDOWN and TRUNC of a number whose 15-digit cut is subnormal return Excel's exact subnormal, so that
probes/t2756-roundup-smallest-normal-threshold A52-A58 pass.

## Context
- T-2756 found that when the 15-digit cut of x is subnormal, ROUNDUP/ROUNDDOWN/TRUNC return that cut whatever
  num_digits (numbers.md "ROUNDUP and ROUNDDOWN of a number whose 15-digit cut is subnormal"). Read back through `*2^100`,
  Excel's value for the cut 2.22507385850720E-308 is 2.2250738585071984E-308, 3 ulps (of 2^-1074) below the nearest
  double 2.2250738585071999E-308 that the engine returns (`round_function` in crates/rustsheet-engine/src/decimal_round.rs).
- Tried and rejected in T-2756: the 15 digits times or divided by two or three exact powers of ten (`1e-k`, `1e+k`, any
  split of 10^-322, each order). The implied scale factor is about 7E-16 relative below 10^k for every split, so the
  power of ten itself is probably inexact (an x87 `2^(y·log2 10)`, or a table entry), or the cut is built by subtraction.
- One cut only is evidenced. Probe more cuts first (other subnormal cuts are reachable only from x in
  [2^-1022, 2.225073858507205E-308), so vary the read-back instead: `*2^k` for several k, and ROUNDDOWN of `-x`).

## Acceptance criteria
- [ ] Probe ROUNDUP/ROUNDDOWN(2^-1022, d) at num_digits outside the evidenced -300..330 (-400, 1E10, -1E10, 2^31):
      does the subnormal cut still win over the `#NUM!` overflow and the digit wrap? Match the engine to it.
- [ ] A rule that reproduces the evidenced subnormal bit-exactly, recorded in numbers.md with its evidence.
- [ ] probes/t2756-roundup-smallest-normal-threshold A52-A58 leave eval/baseline/probes.txt, or the task closes
      time-boxed (decision 0056) with the hypotheses tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The threshold and the flushes (T-2756, done).
