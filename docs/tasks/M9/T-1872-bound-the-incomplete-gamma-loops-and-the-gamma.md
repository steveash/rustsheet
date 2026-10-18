---
id: T-1872
title: "Bound the incomplete-gamma loops and the GAMMA.INV/CHISQ.INV searches, then lift GAMMA.*, CHISQ.* and POISSON (SPEC-0011)"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] Each function above has a kernel with its own per-call cap, entered as `Weighted` with a release measurement of
  its worst case in kinds.rs (or stays out with its reason); its lift cells of probes/stat-argument-modes and
  probes/stat-quantile-lift H27-H50 pass.
- [ ] A test shows the worst-case scalar call of each capped kernel is bounded, `GAMMA.DIST(1E16,1E16,1,TRUE)` included,
  with the values probes-numeric and ironcalc-calc already check unchanged.
- [ ] Each directive-5 cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
