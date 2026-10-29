---
id: T-3034
title: "F.DIST density: port T-2586's frozen model M-final (fdensity_excel.py) where it wins on visible cells"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
`F.DIST(x,d1,d2,FALSE)` follows Excel's own F density structure, T-2586 round 5's frozen model M-final, instead of
the beta density times y·(1-y)/x. The F density's visibly-off cells in probes-numeric fall by roughly the
model's margin over the correctly rounded density (about 14,700 of 64,922 scored cells) with no gated loss.

## Context
- Filed by T-2981 (routing the incomplete-beta sweep cells, eval of 2026-10-28 at `main` 9586958b). The 34 cases
  below hold 37,629 visibly-off F.DIST density cells. No engine task planned to change the F density's structure:
  T-2790 removes the engine's own 1 - y cancellation, T-2219 owns 16 named cells, T-2642 huge df, T-2647 tiny x,
  and T-2586 (research) built the model.
- The rule: docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a GAMMALN formula
  (T-2513)" and its T-2586 round 4 and 5 updates. The model: `docs/research/t2464/r5-f-final/fdensity_excel.py`,
  `value(x, d1, d2, fin="div")`. `rule()` holds the argument and range rules (T-2095 already shipped them),
  `exponent()` builds E = K + fl(c·LN x) - h·L(r), F = fl(EXP(E)/x), with r the smaller of fl(d1·x/d2) and
  fl(d2/(d1·x)), L Excel's log1p (`x87` primitives from T-2645), and the constant from `bcmodel.K_of(m, n, side)`:
  the exact -lnB ± a·ln(m/n) plus a delta table (r3-loader-const's stirlerr e-table and M-sd's bracket). Code must not
  read docs/ (decision 0066): the delta table goes into the engine as data with its provenance cited.
- Scores (r5-f-final/RESULTS.md section 3, bit-exact / within 4 ulp / within 15 digits): all F sets plus the final
  hold-out, 64,922 cells: M-final 12,467/39,629/47,113, CR 4,177/22,038/32,358. On its untouched hold-out
  t2586-r5-f-final-hold: 126/522/722 of 1,584 against CR 63/269/444.
- Where it still misses (section 4 and 6): visible parity only for |c·ln x| < 4 (99-100%), about 87% for |P| in
  [4, 32), 10-36% for |P| >= 32. So the target is a visible-cell gain measured by `make eval`, not parity.
- Engine: `f_pdf_accurate` in `crates/rustsheet-engine/src/functions/statistical/fisher.rs` (beta density times
  y·(1-y)/x where statrs' direct form is finite). Reuse `x87_exp::excel_exp_extended`, `x87_log10::excel_ln` and
  the Excel log1p primitive; stirlerr lives in `functions/statistical/loader.rs` (share it, do not copy it).
- Ordering with T-2790: both change `f_pdf_accurate`. Whichever lands second rebases on the first; if this lands first, T-2790's 224 named t2220 cells are likely settled, so re-triage T-2790 in the Outcome.

### Cases owned (T-2981)
Visibly-off F.DIST density cells per case at the routing eval (each case's F.DIST(...,FALSE) cells; other cells of
the same case are routed to the task named in T-2981's Outcome):
- probes-numeric/t2220-f-dist-sweep-d1-1: F.DIST density 34
- probes-numeric/t2220-f-dist-sweep-d2-1: F.DIST density 60
- probes-numeric/t2220-f-dist-sweep-pdf: F.DIST density 212
- probes-numeric/t2513-r3-fdensity-a: F.DIST density 884
- probes-numeric/t2513-r3-fdensity-hold: F.DIST density 522
- probes-numeric/t2513-r3-fdensity-holdsw: F.DIST density 59
- probes-numeric/t2513-r3-fdensity-sw: F.DIST density 865
- probes-numeric/t2513-r3-tdensity-a1: F.DIST density 115
- probes-numeric/t2513-r3-tdensity-a2: F.DIST density 240
- probes-numeric/t2513-r3-tdensity-hold: F.DIST density 324
- probes-numeric/t2513-verify-r3-fdensity-hold: F.DIST density 435
- probes-numeric/t2586-r4-f-brcomp-big: F.DIST density 1,549
- probes-numeric/t2586-r4-f-brcomp-hold: F.DIST density 742
- probes-numeric/t2586-r4-f-power-a: F.DIST density 4,360
- probes-numeric/t2586-r4-f-power-hold: F.DIST density 1,273
- probes-numeric/t2586-r4-f-small-df-a: F.DIST density 1,872
- probes-numeric/t2586-r4-f-small-df-hold: F.DIST density 543
- probes-numeric/t2586-r5-f-bigconst-a: F.DIST density 3,838
- probes-numeric/t2586-r5-f-bigconst-hold: F.DIST density 1,419
- probes-numeric/t2586-r5-f-combine-a: F.DIST density 3,464
- probes-numeric/t2586-r5-f-combine-hold: F.DIST density 968
- probes-numeric/t2586-r5-f-edges-a: F.DIST density 806
- probes-numeric/t2586-r5-f-edges-hold: F.DIST density 565
- probes-numeric/t2586-r5-f-final-hold: F.DIST density 1,220
- probes-numeric/t2586-r5-f-smallr-a: F.DIST density 5,600
- probes-numeric/t2586-r5-f-smallr-hold: F.DIST density 1,151
- probes-numeric/t2586-verify-r4-f-brcomp-hold: F.DIST density 523
- probes-numeric/t2586-verify-r4-f-power-hold: F.DIST density 805
- probes-numeric/t2586-verify-r4-f-small-df-hold: F.DIST density 104
- probes-numeric/t2586-verify-r5-f-bigconst-hold: F.DIST density 686
- probes-numeric/t2586-verify-r5-f-combine-hold: F.DIST density 701
- probes-numeric/t2586-verify-r5-f-edges-hold: F.DIST density 443
- probes-numeric/t2586-verify-r5-f-final-hold: F.DIST density 567
- probes-numeric/t2586-verify-r5-f-smallr-hold: F.DIST density 680

## Acceptance criteria
- [x] `F.DIST(x,d1,d2,FALSE)` (and every caller of the F density) computes M-final's chain, with unit tests that pin
      bit-exact rows copied from the r5-f-final hold-out into a fixture.
- [x] `make eval`: F.DIST's visibly-off cells in the cases above fall, by at least 10,000 in total, and no gated
      suite loses a cell (`make eval-check`); any region where M-final loses visible cells against today's engine
      keeps today's form, with the comparison in the Outcome.
- [x] statistics.md's F density entry says which form the engine ships and the measured counts.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bit-exactness and the |P| >= 32 power-term noise (T-2586's open item).
- The F cdf and tails (T-2508, T-2510).

## Visible-parity stage
Target (decision 0114): the F density's visibly-off cells in these sweeps drop by the model's measured margin; what
is left is T-2586's open power term.

## Outcome
- `functions/statistical/f_density.rs` `excel_f_density` is M-final (`fdensity_excel.value(x, d1, d2, "div")`) for
  every x > 0 after `f_dist_of`'s rules. Bit for bit the reference model on all 64,883 positive value rows of the
  r5-f-final score sets and hold-out (checked once against the Python model; the 1,549 hold-out rows are a fixture,
  `test/fixtures/f_density_final_hold.fixture`, with 91 bit-exact and 687 within 15 digits of Excel, the RESULTS.md
  126/722 less the 35 rule cells). K's exact part is double-double Stirling (Binet's mu: a 30-entry mpmath table to 15,
  gammaln.rs's Stirling terms above); the offset table (grid_e.tsv, C0 5.33, C_BIG -0.42, e(1/2), e(1), numpy's
  extrapolation slope and intercept) is data in the module with its provenance.
- `make eval`, probes-numeric F.DIST (72,534 cells): visibly-off 38,027 → 18,722 (-19,305); bit-exact 5,595 → 16,268;
  suite passes 359,597 → 370,270. Per cell 21,404 cells move to within 15 digits and 2,591 out. By r5-f-final's
  (side, r <= 0.375, df small/mid/large, |c ln x| < 4 / < 32 / >= 32) regions on the 64,883 research rows, every one of
  the 36 gains (smallest: side 1, r > 0.375, mid df, |P| >= 32, 13 → 19 of 92), so no region keeps the old form.
- Removed: the beta-density-times-Jacobian form, its Loader fallback `f_pdf_loader` and `loader.rs` (stirlerr, bd0,
  dbinom_raw_dev: the F density was its only user; restore from git for T-2507 / T-2585 if they want an accurate
  Loader form). `test_fn_f_pdf_large_df` now checks those cells within 5E-14 relative of the exact density (M-final's
  error there is up to 4.5E-14; Excel's own up to 3E-14), and B889 of t2586-r5-f-bigconst-a within 1E-13 of Excel
  (M-final 1.1626735695122388 against Excel's ...385).
- T-2790 (removing the engine's own 1 - y cancellation in `f_pdf_accurate`) has no code left to change: re-triage it
  against M-final (its 224 named t2220 cells are scored above). T-2219, T-2642, T-2647: their F density cells now go
  through M-final; re-check them with `make eval-show`.
