---
id: T-3525
title: "Re-validate the complex hyperbolic bands (HYPERBOLIC_LOG_CUT, INVERSE_EXP_LOW, INVERSE_EXP_HIGH) on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
The complex hyperbolic bands of engineering/complex/trig.rs (`HYPERBOLIC_LOG_CUT` 709, `INVERSE_EXP_LOW` 0.01, `INVERSE_EXP_HIGH` 20) are shown to generalize on a fresh hold-out sweep (IMSINH, IMCOSH and the functions built on them), or are replaced by the accurate form.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- `HYPERBOLIC_LOG_CUT` is `FITTED(T-2231, 84 cells, |x| 100..800 ...)`; the `1/u` band is
  `FITTED(T-2629, 306 cells, |x| 0.00012..100; cuts bracketed 0.01..0.0105 and 18.2..100)`: the upper cut was fitted
  inside a bracket of 18.2..100, so its placement there is unevidenced. engineering.md (T-2629) has the notes.
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [x] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [x] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [x] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [x] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [x] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).

## Outcome
Both selectors keep their `FITTED` tags, ranges restated (T-3525); no engine behavior changed.

- **Fresh sweeps** (R4.2, oracle-filled on excel-graph, generator `docs/research/t3525/gen_sweep.py`):
  probes-numeric/t3525-im-hyperbolic-fresh-sweep (1,000 cells: IMSINH, IMCOSH, IMSIN, IMCOS, |x| log-uniform in
  [1e-4, 709], and IMSINH/IMCOSH with |x| in [100, 800]) and t3525-im-hyperbolic-log-band-sweep (200 cells, |x| in
  [708.9, 710.6]: the first sweep drew no point in the only window where the log form can differ, so this one was
  added before scoring it).
- **Fit lab** (the method): `rustsheet-fitlab im-hyperbolic` (old cells) and `im-hyperbolic-fresh`, new in
  `kernels/im_hyperbolic.py`. The lab now scores IM* text part by part (`Fit.complex_text`) and takes other harness
  builds as candidates (`--build NAME=PATH`). Builds: `no-band` (`in_inverse_exp_band` always false: libm, or the
  EXP pair in IMSIN/IMCOS, everywhere) and `no-log-form` (`hyperbolic_log_form` never applies). Hold-out split
  T-3360, margin 5 points, reference `correctly rounded` (50-digit mpmath, parts rounded to 15 digits).

| sweep | candidate | train exact | test exact | verdict |
|---|---|---|---|---|
| old (1,359; 965 train / 394 test) | engine (selectors) | 965 (100%) | 394 (100%) | accept |
| | no-band | 805 (83.4%) | 310 (78.7%) | accept, below engine |
| | no-log-form | 923 (95.6%) | 382 (97.0%) | accept, below engine |
| | correctly rounded | 741 (76.8%) | 291 (73.9%) | reference |
| fresh (1,200; 850 / 350) | engine (selectors) | 815 (95.9%) | 336 (96.0%) | accept |
| | no-band | 798 (93.9%) | 324 (92.6%) | accept, below engine |
| | no-log-form | 686 (80.7%) | 289 (82.6%) | accept, below engine |
| | correctly rounded | 603 (70.9%) | 255 (72.9%) | reference |

  The selectors' test rate is within the margin of train on both sweeps and beats the accurate form on test and
  on the fresh sweep, so both stay.
- **By |x|** (exact, engine / no-band / no-log-form). 1/u band: old 0.0105..18.2 1214/1214 vs 970; fresh 388/388 vs
  361; fresh 0.01..0.0105 4/4 vs 2. Cut placement: no cell in 0..0.01 (220) or 18.2..100 (110) changes between the
  two builds, so neither cut is pinned further; the bracket 0.01..0.0105, 18.2..100 stands. Log form: old
  709..710.5 75/75 vs 21; fresh 709..710.5 178/180 vs 2.
- **Restated tags (R4.3)**: `HYPERBOLIC_LOG_CUT` `FITTED(T-3525, 617 cells, |x| 100..800 …; cut bracketed
  709..709.0000000001 for x > 0)`; `INVERSE_EXP_LOW`/`HIGH` `FITTED(T-3525, 1942 cells, |x| 0.0001..100; cuts
  bracketed …)`. Both apply only inside their measured ranges. `scripts/check_fitted.py`: 86 tags, unchanged.
- **Gate A envelope (R2)** on the first fresh sweep: all 1,000 cells within 1e-9 relative of 50-digit mpmath (988;
  worst 4.2e-13) or equal to Excel (12, all Excel's own `#NUM!`).
- **Not matching**: fresh sweep 961/1000, log band 190/200. Findings recorded in docs/excel-semantics/engineering.md:
  (1) for x < 0 Excel takes the log form below |x| = 708.94, not past 709 (8 log-band cells), T-3546;
  (2) IMSIN and IMCOS with 1e-4 < |y| < 0.0085 are neither the EXP pair nor exact (34 cells), T-3545;
  (3) 3 IMSINH/IMCOSH cells below |x| = 0.01 miss libm (noted in T-3545's out-of-scope line, not filed:
  too few cells to fit without a dedicated probe).
- **Eval delta**: none for gated suites (no engine change). probes-numeric (not gated) gains 1,200 cells, 1,151
  passing.
- Follow-ups: T-3545, T-3546.
