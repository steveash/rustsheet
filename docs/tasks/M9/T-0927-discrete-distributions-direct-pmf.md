---
id: T-0927
title: "BINOM.DIST/NEGBINOM.DIST/BINOM.DIST.RANGE/POISSON.DIST: direct pmf evaluation (~140 known failures)"
milestone: M9
lane: functions-stat
status: done
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
- [x] No panics for huge n/k (bounded loops; large-n CDF falls back to a bounded method); unit tests for n = 0, p in {0,1}.
- [x] `make check` green.

## Out of scope
BINOM.INV (CRITBINOM), GAMMA.DIST/BETA.DIST kernels.

## Outcome
Partial: the binomial half is done, and POISSON.DIST moves to T-2042.
`functions/statistical/discrete.rs` sums the binomial pmf terms in double-double with a power-of-two exponent. Each
term comes from the last by the exact ratio of consecutive terms, and there is one rounding at the end. A walk is
capped at 2^10 steps, with statrs as the fallback past it. BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST (as a
binomial of f + r trials) now give the exact sum rounded. The five binomial functions are charged 4096 units an
element in the lifting table (BINOM.INV's weight, a DEVIATION past 32,768 elements).
POISSON.DIST: a model fitted to ironcalc-calc POISSON.xlsx, `fl(fl(S)*exp(-mean))`, broke 8 excel-graph cells of
probes-numeric/t1048-poisson that statrs matches. Routed into even-df CHISQ.DIST.RT, it gave 1.0000000000000002 where
Excel gives 1.0. The Excel-fidelity review caught both, so POISSON.DIST stays on statrs; the evidence is in
docs/excel-semantics/statistics.md "Discrete distributions: within an ulp of the exact sum", and follow-up T-2042 owns
it.
Eval: ironcalc-calc known failures 1113 -> 1072 (41 fixed, no regressions; BINOM.xlsx 111 -> 70 lines). Ungated
probes-numeric m9-legacy-binomdist and t1048-negbinomdist only gain passing cells. Residue, all listed in the
semantics entry:
- 13 NumericClose binomial tails where Excel is more than 4 ulp from the exact value. The cause is its incomplete-beta
  prefactor `exp(a*ln x)`, so they need Excel's incomplete-beta kernel (T-1744's fit, T-0970). The cells are
  BINOM.DIST E24, E32, E67; BINOM.DIST.RANGE E10, E21, E23; NEGBINOM.DIST E3, E26, E30, E37, E40, E74, E75.
- 57 NumericUlp cells, where Excel is within 4 ulp of the exact value but not equal to it.
- POISSON.xlsx, 32 lines, unchanged (T-2042).
