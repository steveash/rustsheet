---
id: T-2146
title: "BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST last digits: fit Excel's evaluation (70 ironcalc-calc BINOM.xlsx known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
campaign: visible
depends_on: [T-2145]
---

## Goal
BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST and NEGBINOMDIST give Excel's doubles. T-0927's residue in
`ironcalc-calc/STATISTICAL/BINOM.xlsx` has no open owner today: T-2055 lists the binomial family as out of scope
and points back to T-0927, which is done.

## Context
- The oracle sweep (T-2145) is landed: probes-numeric/t2145-binomial-sweep, t2145-negbinom-sweep
  and t2145-binom-range-sweep (statistics.md "Discrete distributions"). T-0927 tried and ruled out the exact sum, Loader's saddle-point pmf (R's
  `dbinom_raw`: 49 of 122) and `exp(a*ln x)` times the exact remaining factor, all on the cached cells alone.
- Known failures (eval/baseline/ironcalc-calc.txt, gap triage 2026-10-20): 70 lines, BINOM.xlsx sheet by sheet:
  - BINOM.DIST: 18 (15 NumericUlp, for example E3: Excel 0.23925373020405763, ours ...576; 3 NumericClose:
    E24, E32, E67, for example E24 `BINOM.DIST(2,10,0.999999,TRUE)`: Excel 4.499992001038836E-47, ours ...802E-47).
  - BINOM.DIST.RANGE: 17 (14 NumericUlp, for example E2: 0.2552983299999998 against ...999; 3 NumericClose: E10,
    E21, E23).
  - NEGBINOM.DIST: 35 (28 NumericUlp, for example E4: 0.02800000000000001 against 0.028000000000000004; 7
    NumericClose: E3, E26, E30, E37, E40, E74, E75).
  - Ungated: probes-numeric/m9-legacy-binomdist (21 failing, for example A5 `BINOMDIST(3,10,0.5,TRUE)`:
    0.17187500000000006 against the exact 0.171875) and t1048-negbinomdist (12 failing, for example A7
    `NEGBINOMDIST(3,3,0.75)`: 0.06591796875000003 against the exact 0.06591796875).
- Evidence and hypotheses (statistics.md "Discrete distributions: within an ulp of the exact sum"):
  `NEGBINOM.DIST(0,100,0.01,TRUE)` is `exp(100*ln(0.01))` in doubles bit for bit, Excel's BETA.DIST prefactor
  ("Incomplete beta: Excel's closed forms and the best candidate", T-1744). So a likely shape for the cdf is the
  regularized incomplete beta, `BINOM.DIST(k,n,p,TRUE) = I_{1-p}(n-k, k+1)`, with Excel's own prefactor. The pmf may be
  `exp` of a log-gamma sum, or a ratio. Try them in the fit lab (`cd tools && uv run rustsheet-fitlab`,
  docs/architecture/fit-lab.md). If the cdf is the incomplete beta, share `statistical/incomplete_beta.rs` (T-0970;
  T-2119 moves BETA.DIST onto it), not a second kernel.
- Code: `crates/rustsheet-engine/src/functions/statistical/discrete.rs` (T-0927's exact walk, capped at 2^10 steps
  with statrs past it).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 14 cells, 14 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on NEGBINOM.DIST 6, BINOM.DIST 4, BINOMDIST 2, NEGBINOMDIST 2.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| ironcalc-calc/STATISTICAL/BINOM.xlsx BINOM.DIST!E24 | `=_xlfn.BINOM.DIST(A24,B24,C24,D24)` | 4.499992001038836e-47 | 4.499992001038802e-47 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx BINOM.DIST!E32 | `=_xlfn.BINOM.DIST(A32,B32,C32,D32)` | 9.90100000000073e-197 | 9.90100000000087e-197 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E3 | `=_xlfn.NEGBINOM.DIST(A3,B3,C3,D3)` | 6.50284892159999e-10 | 6.502848921600006e-10 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E30 | `=_xlfn.NEGBINOM.DIST(A30,B30,C30,D30)` | 6.599988000005536e-59 | 6.599988000005497e-59 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E37 | `=_xlfn.NEGBINOM.DIST(A37,B37,C37,D37)` | 1.0000000000000348e-200 | 1.000000000000002e-200 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E40 | `=_xlfn.NEGBINOM.DIST(A40,B40,C40,D40)` | 1.690659363904351e-142 | 1.6906593639043344e-142 |
| probes-numeric/m9-legacy-binomdist Calc!A33 | `=BINOMDIST(2,5,0.999,TRUE)` | 9.985006000000038e-09 | 9.985006000000026e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A34 | `=BINOM.DIST(2,5,0.999,TRUE)` | 9.985006000000038e-09 | 9.985006000000026e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A35 | `=BINOMDIST(2,5,0.999,FALSE)` | 9.980010000000044e-09 | 9.980010000000027e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A36 | `=BINOM.DIST(2,5,0.999,FALSE)` | 9.980010000000044e-09 | 9.980010000000027e-09 |
| probes-numeric/t1048-negbinomdist Calc!A4 | `=NEGBINOMDIST(10,2,0.9)` | 8.909999999999952e-10 | 8.909999999999981e-10 |
| probes-numeric/t1048-negbinomdist Calc!A5 | `=NEGBINOMDIST(1,10,0.1)` | 8.999999999999998e-10 | 9.000000000000005e-10 |
| probes-numeric/t1048-negbinomdist Calc!B4 | `=NEGBINOM.DIST(10,2,0.9,FALSE)` | 8.909999999999952e-10 | 8.909999999999981e-10 |
| probes-numeric/t1048-negbinomdist Calc!B5 | `=NEGBINOM.DIST(1,10,0.1,FALSE)` | 8.999999999999998e-10 | 9.000000000000005e-10 |

## Acceptance criteria
- [ ] Visible parity: the 14 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 6 lines in eval/baseline/ironcalc-calc.txt disappear (`make eval-baseline`); the 8 cells of the ungated probes-numeric suite pass in `make eval`, with no new known-failure lines in any baseline.
- [ ] statistics.md "Discrete distributions" records the fit: each candidate's count on BINOM.xlsx's 148 numeric
      cells, on T-2145's sweep and on the two probes-numeric cases, including the candidates that lost.
- [ ] The formulation that wins is implemented. The BINOM.xlsx known-failure lines it fixes disappear from
      eval/baseline/ironcalc-calc.txt, with no new ones in any gated suite. The target is all 70 lines; say which
      ones remain and why.
- [ ] No BINOM.INV/CRITBINOM cell regresses (they search on the same cdf).
- [ ] `make check` green; reviewers approve.

## Out of scope
- POISSON.DIST (T-2055), HYPGEOM.DIST (T-0977, T-1896), BINOM.INV with probability_s 0 (T-2050).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (binomial and negative-binomial values that differ in the 14th-15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083). The fit needs T-2145's oracle sweep first.
