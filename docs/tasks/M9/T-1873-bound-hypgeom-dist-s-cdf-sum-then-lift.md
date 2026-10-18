---
id: T-1873
title: "Bound HYPGEOM.DIST's cdf sum, then lift HYPGEOM.DIST and HYPGEOMDIST (SPEC-0011)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST lift every argument over arrays and ranges like the other statistical functions, once
HYPGEOM.DIST's cdf sum has a per-call cap of its own.

## Context
Split from T-1848 (statistics.md "Quantile searches: per-call cap and lifting"). statrs 0.18's `Hypergeometric::cdf`
sums the pmf term by term from 0 to x, three `ln_binomial`s a term:
`HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` takes about 4 s on main. Terms below `max(0, n + K - N)` are
exactly 0 and terms far from the mode underflow to 0, so a bound that keeps today's values may skip them; past that a
term cap is a DEVIATION. The pmf (`cumulative` FALSE, HYPGEOMDIST) is closed form. HYPGEOMDIST builds a fifth argument
node today (`fn_hypgeomdist_compat`); a kernel takes the four values instead.
Evidence: probes/stat-argument-modes H330-H342 (every argument over `{v;v}`, the first over a range);
probes/stat-quantile-lift H51-H54 (the last argument over a two-cell range is 2 rows, values in I and J; excel-graph,
2026-10-18). Probe a large population first if a cap changes a result.

## Acceptance criteria
- [x] HYPGEOM.DIST's cdf has a per-call cap of its own and is entered as `Weighted` with a release measurement of its
  worst case in kinds.rs; HYPGEOMDIST is `Constant` if its kernel is the closed-form pmf; their lift cells of
  probes/stat-argument-modes and probes/stat-quantile-lift H51-H54 pass.
- [x] A test shows the worst-case scalar call is bounded, with the values probes-numeric and ironcalc-calc check unchanged.
- [x] A cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done. statrs 0.18's `Hypergeometric::cdf` is ported to `statistical/hypegeom.rs` (MIT notice there and in NOTICE): the
same terms in the same order, skipping only terms that are exactly 0 (below max(0, n + K - N), and, at a population up
to 2^52, those whose computed exponent is below -750 by more than twice a bound on its rounding error, found by a
bisection on each side of the mode, at most 130 exponents), and NaN (`#NUM!`) once more than 2^18 terms are left. Adding
+0.0 leaves the sum unchanged, so within the cap the value is statrs' double bit for bit: unit tests compare 3,000
random small cdfs and 300 cdfs that go through the skip (200 of them kept, 100 past a test cap of 2^12) with statrs
bit for bit, and `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` (4.8 s on main, 25,000,001 terms) now sums
96,259 terms to main's 0.5000797091291159. The cap keeps every population up to about 1.84E8 whatever the sample and x
(261,091 terms at the widest distribution there). `the_cap_counts_the_terms_summed` drives both sides of the cap (with
nothing skipped and after the skip). HYPGEOM.DIST is `Lift` and `Weighted(2^22)`: a term costs 167 to 193 ns in release
(six Lanczos `ln_gamma`s), so the bound is (2^18 + 130) x 193 ns, about 51 ms an element, and 32 elements about 1.6 s;
measured, 32 elements at the widest distribution of a population of 1.8E8 (258,239 terms each) took 1.39 to 1.48 s
(benches/lift.rs). `hypgeom_dist_is_charged_its_weight` tests 32 and 33 elements. HYPGEOMDIST is `Lift` and `Constant`
(the closed-form pmf), one kernel of the four counts with HYPGEOM.DIST instead of the fifth-argument-node wrapper.
Eval: probes 15 known failures fixed (stat-argument-modes H331-H336, H338-H342; stat-quantile-lift H51, H53, J51,
J53), 0 regressions, 0 new known failures; probes, probes-numeric, ironcalc-calc and gen-m9 unchanged cell for cell
against main otherwise (the scalar H330 and H337 keep main's last-digit residues, T-0977). static_analysis.rs
1146 -> 1141 lines (the 4 fallback lines now in the lifting table); duplication unchanged at 3989. Scalar change:
`#NUM!` past the cap where main gave a number after minutes
(`HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)`, main 1.0000065293398746 after 191 s). Two
DEVIATIONs in statistics.md "HYPGEOM.DIST: per-call cap and lifting". Follow-up: T-1896 (Excel's values past the cap).
