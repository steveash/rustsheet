---
id: T-1872
title: "Bound the incomplete-gamma loops and the GAMMA.INV/CHISQ.INV searches, then lift GAMMA.*, CHISQ.* and POISSON (SPEC-0011)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV,
POISSON and POISSON.DIST lift every argument over arrays and ranges like the other statistical functions, once their
incomplete-gamma loops and the GAMMA.INV/CHISQ.INV search have a per-call cap of their own.

## Context
Split from T-1848, which bounded and lifted the beta/F/t quantile searches (statistics.md "Quantile searches:
per-call cap and lifting"; `statistical/inverse_beta.rs` is the pattern: a line-for-line port of the statrs loop with
caps, bit for bit statrs where it finishes). SPEC-0011's security rule: a kernel whose loop length depends on its
argument values must be `KernelWork::Weighted` and keep a per-call cap of its own.
- statrs 0.18's regularized incomplete gamma (`gamma_lr`, `gamma_ur`: a series and a continued fraction run to a 1E-15
  tolerance with no iteration cap) is behind GAMMA.DIST, CHISQ.DIST*, CHIDIST and POISSON's cdf. Measured on main in
  release: `GAMMA.DIST(1E10,1E10,1,TRUE)` 5.9 ms, `GAMMA.DIST(1E12,1E12,1,TRUE)` 24 ms, `POISSON(1E12,1E12,TRUE)` 16 ms,
  `CHISQ.DIST(1E10,1E10,TRUE)` 1.3 ms (df is capped at 1E10), and `GAMMA.DIST(1E16,1E16,1,TRUE)` does not finish (once
  a + 1 == a the series no longer converges). GAMMA.DIST's alpha and POISSON's x and mean have no upper limit here.
- statrs' `Gamma::inverse_cdf` (GAMMA.INV, CHISQ.INV*) halves and doubles a bracket (each up to about 1,075 cdfs, each
  cdf the loop above), then 8 bisections and 4 Newton steps. Today `GAMMA.INV(0.5,1E10,1)` and `CHISQ.INV(0.5,1E10)` are
  `#NUM!` (Excel unprobed).
- Evidence: probes/stat-argument-modes H132-H156, H293-H314, H476-H485 (every argument over `{v;v}`, the first over a
  range); probes/stat-quantile-lift H27-H50 (the last argument over a two-cell range is 2 rows, with Excel's element
  values in I and J; excel-graph, 2026-10-18). Probe the edge values a cap changes (huge alpha, x near alpha) first.

## Acceptance criteria
- [x] Each function above has a kernel with its own per-call cap, entered as `Weighted` with a release measurement of
  its worst case in kinds.rs (or stays out with its reason); its lift cells of probes/stat-argument-modes and
  probes/stat-quantile-lift H27-H50 pass.
- [x] A test shows the worst-case scalar call of each capped kernel is bounded, `GAMMA.DIST(1E16,1E16,1,TRUE)` included,
  with the values probes-numeric and ironcalc-calc already check unchanged.
- [x] Each directive-5 cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done. statrs 0.18's `checked_gamma_lr`, `checked_gamma_ur` and `Gamma::inverse_cdf` are ported line for line to
`statistical/incomplete_gamma.rs` (MIT notice there and in NOTICE) with one cap of 2^19 series or continued-fraction
terms per cdf and per whole inversion (its cdfs share the budget), bit for bit statrs wherever statrs finishes within it
(unit sweep `the_port_is_statrs_where_statrs_finishes`), NaN (`#NUM!`) where statrs hung (`GAMMA.DIST(1E16,1E16,1,TRUE)`,
`POISSON(1E16,1E16,TRUE)`, a shape of 1E300, a NaN continued fraction) or panicked where `x / beta` overflows (Excel
`#NUM!` too); where it underflows to 0 (statrs panicked) the cdf is statrs' value at its smallest x, 0, as in Excel
(probe E7). The cap keeps every chi-squared cdf (481,117 terms at df 1E10) and, in 3 million sampled
inversions, every inversion statrs finished with a shape under 3.5E8. Unit test `a_call_past_its_cap_is_nan` drives
both sides of each cap (481,117 / 481,116 series terms, the fraction's own count, 675,286 at a = 1E10, and an
inversion's 476,534 / 476,533). The twelve functions (CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, CHISQ.INV, CHISQ.INV.RT, CHIINV,
GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON, POISSON.DIST) are `Lift` entries, all `Weighted(2^18)`: measured in
release over 512 elements at the cap, a continued-fraction term costs 4.1 to 4.7 ns on its NaN path
(`GAMMA.DIST(1.0000001E300,1E300,1,TRUE)`, the slowest) and 2.4 ns where it converges and rescales at the cap
(`GAMMA.DIST(1000000000000001,1E15,1,TRUE)`, bench min 653 ms), a series term 2.5 ns and a cdf's own work 49 ns, so the bound is 2^19 x 4.7 ns + 2,120 cdfs x 49 ns, about 2.6 ms an element, and 512 elements about
1.3 s (`incomplete_gamma_kernels_are_charged_their_weight` tests 512 and 513 for each entry shape). CHISQ.TEST uses the
same capped cdf. New probe probes/incomplete-gamma-cap (38 cells, excel-graph): Excel's values where the cap or a statrs
hang/panic gives `#NUM!` (18 known failures, all listed in T-1891's acceptance criteria), the L element cells all pass. Eval: probes 68 known
failures fixed (stat-argument-modes 45, stat-quantile-lift 22, incomplete-gamma-cap E7), 0 regressions; probes-numeric, ironcalc-calc and every
other suite unchanged cell for cell against main; duplication 4248 -> 4085, static_analysis.rs 1181 -> 1150 (this PR removes 24 lines; main's file was 1174, so the lock was 7 lines loose).
Scalar changes: `#NUM!` instead of main's (inaccurate) number past the cap (`GAMMA.DIST(1E10,1E10,1,TRUE)`, POISSON at a
mean of 1E10, CHISQ.INV for 6 of 1,960 sampled calls at df 1E8 to 1E10), and instead of a hang or panic; CHISQ.TEST of a
statistic of 5E-324 is `#NUM!` where main panicked (Excel unprobed), and one whose shape passes about 5.9E9 with its
statistic near df is `#NUM!` (the cap). Two DEVIATIONs in statistics.md "Incomplete gamma: per-call cap and lifting". Remaining
scalar last-digit misses beside the lift cells are listed in T-0971 and T-0972. Follow-up: T-1891 (Excel's values at a
huge shape and where statrs panicked).

