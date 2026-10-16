---
id: T-0927
title: "BINOM.DIST/NEGBINOM.DIST/BINOM.DIST.RANGE/POISSON.DIST: direct pmf evaluation (~140 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Discrete distributions match Excel's cached values by evaluating the pmf (and CDF as a sum of pmf terms) directly instead of via statrs' incomplete-beta/gamma CDF.

## Context
Kernel map (T-1522, decision 0056): rank 3 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Related fit: T-1744 (incomplete beta) may help BINOM.DIST cumulative, but this task does not wait for it.
Oracle-free: Excel's values are exact where the maths is exact, ours carry ~1e-14 relative error from the regularised-incomplete-function route.
- `ironcalc-calc/STATISTICAL/BINOM.xlsx` `BINOM.DIST!E2` `=BINOM.DIST(7,8,0.7,TRUE)` expected 0.94235199 (exact decimal), actual 0.9423519899999973; `E5` expected 0.0546875 actual 0.05468750000000168; `NEGBINOM.DIST!E4` expected 0.02800000000000001 actual 0.027999999999999938. Counts: BINOM.DIST 41, NEGBINOM.DIST 42, BINOM.DIST.RANGE 28 (111 baseline lines in the file).
- `ironcalc-calc/STATISTICAL/POISSON.xlsx` `POISSON.DIST!D2` expected 0.12465201948308113 actual 0.12465201948308068; 32 cells.
Code: `functions/statistical/{binom,poisson}.rs` (statrs `Binomial`, `NegativeBinomial`, `Poisson`).
Approach to try: pmf via a stable product / log-free recurrence (e.g. C(n,k) p^k q^(n-k) built multiplicatively, Poisson via exp(-mean) * mean^k/k! recurrence), CDF as the ascending sum of pmf terms. Iterate against the cached cells; truncation rules for non-integer arguments already pass and must stay. Record which formulation matched in `docs/excel-semantics/`.

Triage T-0903 (2026-10-06 report): this task owns 143 baseline lines (BINOM.xlsx 111: BINOM.DIST 41,
NEGBINOM.DIST 42, BINOM.DIST.RANGE 28; POISSON.xlsx 32). If pmf summation does not fit, the accurate
incomplete beta (T-0970) and incomplete gamma (T-0971) kernels are the alternative CDF route.
HYPGEOM.DIST is separate (T-0977): Excel is not exact there, so a direct product does not fit it.

## Acceptance criteria
- [ ] All NumericClose lines for BINOM.DIST, NEGBINOM.DIST, BINOM.DIST.RANGE, POISSON.DIST disappear from `eval/baseline/ironcalc-calc.txt` (state exact counts; ulp-only residue may remain only if listed in the PR with expected/actual), no new lines.
- [ ] No panics for huge n/k (bounded loops; large-n CDF falls back to a bounded method); unit tests for n = 0, p in {0,1}.
- [ ] `make check` green.

## Out of scope
BINOM.INV (CRITBINOM), GAMMA.DIST/BETA.DIST kernels.
