# Fit audit: thinly evidenced gamma-family FITTED constants (T-3530)

Decision 0188 R4 audit of the gamma-family `FITTED` constants with 6 or fewer evidence cells, on `main` 744815d06.
Evidence: probes-numeric `fit-audit-*` (excel-graph, 2026-10-31), the engine at 744815d06, and 50-digit mpmath
(regularised incomplete gamma; positive-term lower series in the far left tail at shapes of 1E6 and up).

**Status:** the type-level rules (where Excel's result is a number, 0 or `#NUM!`) have verdicts, re-scored on `main`
4e2f5f3df (after T-3350) in the section "Re-score on main" just below; the wrong-type cells went to their owning tasks. The
six bit-level constants marked *provisional* are T-3633's (two-sided scoring with a scratch build). No engine code
changes on this branch.

## Re-score on main 4e2f5f3df (after T-3350)

T-3350 replaced `BAND_LOWER_EDGE` (`offset - z < 39`) with a cdf-ratio edge (`ln P(a, x0) - ln p < 205.3`) after
the table above was scored, so every case was re-scored on main with a release build. Counts: `=` bit-exact, `4u`
within 4 ulp, `15d` within 15 digits, `off` beyond, `err` a different type or error.

| Case | n | = | 4u | 15d | off | err | Wrong-type cells → owner |
|---|---|---|---|---|---|---|---|
| fit-audit-t3271-density-cut | 50 | 47 | 0 | 0 | 0 | 3 | A9, A43, A48 → T-3316 |
| fit-audit-t3271-cdf-cut | 38 | 17 | 0 | 0 | 0 | 21 | A4, A6, A7, A9–A11, A13–A17, A40 (Excel `#NUM!`, ours 0); A28, A29, A32, A34–A39 (Excel 0, ours `#NUM!`) → T-3316 |
| fit-audit-t3070-shape-min | 39 | 16 | 14 | 0 | 1 | 8 | A19, A22, A23, A25, A26, A28–A30 → T-3409 |
| fit-audit-t3070-band-upper | 52 | 25 | 11 | 0 | 0 | 16 | A1–A3, A5–A7, A12, A13, A26, A32, A33, A38, A39 → T-3409; A47–A49 → T-3396 |
| fit-audit-t3070-band-lower | 42 | 33 | 9 | 0 | 0 | 0 | none: T-3350's edge separates every cell |
| fit-audit-t3070-closed-form-cut | 59 | 31 | 22 | 6 | 0 | 0 | none (verdict d confirmed) |
| fit-audit-t2638-ratio-min | 62 | 11 | 4 | 1 | 46 | 0 | the region → T-3617; the cut → T-3633 |
| fit-audit-t2852-shape-max | 52 | 13 | 35 | 0 | 4 | 0 | T-3633 |
| fit-audit-t2852-shape-min | 48 | 12 | 36 | 0 | 0 | 0 | T-3633 |
| fit-audit-t3013-quantile-max | 54 | 20 | 7 | 9 | 18 | 0 | T-3633, T-3265 |
| fit-audit-t3013-step-min | 20 | 11 | 1 | 0 | 8 | 0 | T-3633, T-3265 |
| fit-audit-t3147-ratio-shape | 56 | 15 | 36 | 2 | 3 | 0 | T-3633 |

- **`BAND_LOWER_EDGE`**: no longer refuted. The refutation above was of the old `offset - z < 39` rule.
- **`HUGE_SHAPE_CDF_CUT`**: refuted both ways, and Excel's cut is not monotone in the shape. At x/beta = 2E-10 (x = 2,
  beta = 1E10) Excel is `#NUM!` from shape 1E60 (A4) through 1.8E76, except 0 at 1E76 (A12) and 1.84E76 (A18). At
  beta = 1 it is 0 at shapes from 1E60 to 1E200 for x = 1 and x = 1E50, and for x = 1E-100 it is 0 except at 1.85E76
  (A30, `#NUM!`). A refill of A4, A12 and A18 in fresh workbooks gave the same values (`refill` in
  `cdf-cut-slices.json`). No rule in the shape alone fits. The `#NUM!`
  probably comes from an intermediate in Excel's arithmetic (UNVERIFIED). excel-graph refused the 42-cell workbook,
  so each cell was filled in its own workbook. A5, A8, A31 and A33 failed at createSession on both attempts each and have
  no value. The
  per-cell sources are in `cdf-cut-slices.json`, and the oracle sidecar's source names that file.

## First scoring, on 744815d06 (superseded)

Branches: T-3311 (PR #2599) moves `CLOSED_FORM_QUANTILE_MAX` and `CLOSED_FORM_STEP_MIN` to `closed_form.rs`
without changing them. T-3285 and T-3269 add new FITTED constants but change none of these.

**The table and "Findings so far" below are the first scoring, on 744815d06 before T-3350, and are
superseded where they differ from the section "Re-score on main" above**, notably on `BAND_LOWER_EDGE` (no longer refuted)
and the counts. They are kept as the record of what the audit found first.

Notation: "exact" means bit-exact, "≤15" means within 15 significant digits but not bit-exact, "off" means numeric and
beyond 15 digits, "err" means Excel's error differs from the engine's value (or the reverse).

| Constant (file, old tag) | Old evidence | New evidence (case, cells) | Verdict | Change |
|---|---|---|---|---|
| `HUGE_SHAPE_DENSITY_CUT` 6.9187E11 (gamma.rs:36, T-3271, 3 cells) | t2435 A26 `#NUM!`, A92 and A31 0 | fit-audit-t3271-density-cut, 50: 46 agree, 4 err | **(b) refuted** | T-3316 |
| `HUGE_SHAPE_CDF_CUT` 1.85E76 (gamma.rs:44, T-3271, 2 cells) | t2435 A100 0, A101 `#NUM!` | fit-audit-t3271-cdf-cut, 38 of 42 filled one cell per workbook: 17 agree, **21 err** | **(b) refuted** | T-3316 |
| `LOG_FORM_SHAPE_MAX` 3.0 (gamma.rs:67, T-2852, 6 cells) | t1049, gen-m9, m9-chisq-dist, m9-gamma-dist-a9 | fit-audit-t2852-shape-max, 52: 13 exact, 35 ≤15, 4 off (A25, A29 at 3 and 3.01; A34 3.2; A46 4.5) | provisional: not scored on both sides | none |
| `LOG_FORM_SHAPE_MIN` 0.5 (gamma.rs:84, T-2852, 2 cells) | gen-m9 A367, A386 | fit-audit-t2852-shape-min, 48: 12 exact, 36 ≤15, 0 off | provisional (a) candidate: no cell off on either side; bit-level split not scored | none |
| `HUGE_SHAPE_MIN` 1E9 (huge_shape.rs:17, T-3070, 1 cell) | t3070 A1 | fit-audit-t3070-shape-min, 39: 29 agree, 9 err, 1 off | **(b) refuted** | T-3409 |
| `BAND_UPPER_EDGE` 27.25 (huge_shape.rs:74, T-3070, 2 cells) | t3070 A77, t2982 A45 | fit-audit-t3070-band-upper, 52: 37 agree, 10 err, 5 off | **(b) refuted** | T-3409, T-3396 |
| `BAND_LOWER_EDGE` 39 (huge_shape.rs:83, T-3070, 3 cells) | t2982 A44, t3070 A44, A370 | fit-audit-t3070-band-lower, 42: 24 agree, 12 err, 6 off | refuted on 744815d06; **holds after T-3350** (0 err, re-score) | none |
| closed-form density cut, `f64::MIN_POSITIVE` (huge_shape.rs:103, T-3070, 2 cells) | t3070 A110, A210 | fit-audit-t3070-closed-form-cut, 59: 31 exact, 28 ≤15, 0 off, 0 err; ln density at the start from -760 to -695, shapes 1.3E10 to 2.3E10 | **(d) Excel's own boundary**: the closed form holds exactly where the density at the start is below the smallest normal double | Tag to be narrowed and its count raised to 61 (not done: the tag edit goes through `check_fitted.py --update` with `make check`) |
| `RATIO_FORM_MIN` 0.005 (incomplete_gamma/mod.rs:253, T-2638, 6 cells) | gen-m9 A333, A341; t2753 A47, A48, B47, B48 | fit-audit-t2638-ratio-min, 62: 11 exact, 5 ≤15, **46 off** (Excel within 8.3E-14 of the math) | provisional: the large-shape density at x/a 0.002 to 0.03 is beyond 15 digits on both sides of the cut. Whether the cut matters cannot be said until the two-sided score is run | none; likely its own task |
| `CLOSED_FORM_QUANTILE_MAX` 1E-21 (mod.rs:424, T-3013, 5 cells) | t3012 A18, A28, A34, B18; gen-m9 A210 | fit-audit-t3013-quantile-max, 54: 20 exact, 16 ≤15, 18 off | provisional; evidence for T-3265 | none (T-3265 owns it) |
| `CLOSED_FORM_STEP_MIN` 3.5E-14 (mod.rs:444, T-3013, 4 cells) | t3012 A6, A26; gen-m9 A254, A396 | fit-audit-t3013-step-min, 20 (steps 2.6E-14 to 4.4E-14 on the engine's model): 11 exact, 1 ≤15, 8 off (A10–A18, steps 3.4E-14 to 4.2E-14) | provisional: off cells straddle the cut, so the step model, not just the cut, is in question; evidence for T-3265 | none (T-3265 owns it) |
| `SMALL_RATIO_SHAPE_MIN` 3.0 (density.rs:16, T-3147, 3 cells) | gen-m9 A42, m9-chisq-dist A50, t1049 A46 | fit-audit-t3147-ratio-shape, 56: 15 exact, 37 ≤15, 4 off (A12 2.7, A15 2.8, A28 2.99, A41 3.3) | provisional: not scored on both sides | none |

## Findings so far

- **GAMMA.DIST density at a huge shape (`HUGE_SHAPE_DENSITY_CUT`)**: `(shape-1)·(-LN(x/beta))` is not Excel's
  variable. At or past 6.9187E11 Excel is 0 at shapes 1E10 to 1E15 (A16–A44, e.g. A43
  `GAMMA.DIST(0.5,1E+15,1,FALSE)` = 0, where the engine gives `#NUM!`). Below it Excel is `#NUM!`
  (A9 `GAMMA.DIST(2.17173678289098E-300,1E+9,1,FALSE)`, product 6.9E11; A48 `GAMMA.DIST(1E-300,1E+9,1E-6,FALSE)`,
  product 6.77E11), where the engine gives 0. The scale does not count: A45 `GAMMA.DIST(1E-200,1E+9,1E+100,FALSE)`
  (x/beta = 1E-300) is 0. At the probed shapes 1E8 to 1E15 every cell, these and t2435 A25, A26, A92 included, fits a
  rule on x / shape: `#NUM!` where x/shape is below a cut in (2.17E-309, 1E-308], 0 above it (UNVERIFIED: it needs a
  second probe at smaller shapes and x/shape between those values). It does not explain cdf-cut A41 and A42
  (`GAMMA.DIST(2,1.85E+76,1E+10,FALSE)`, `GAMMA.DIST(2,1E+70,1E+10,FALSE)`), `#NUM!` at x/shape about 1E-76 and 2E-70.
- **GAMMA.INV `HUGE_SHAPE_MIN`**: Excel is already `#NUM!` from shape 6E8 at p = 1E-300, 7E8 at 1E-250 and 9E8 at
  1E-200 (A19, A22, A23, A25, A26, A28–A30; A20, A21, A24 and A27 are numbers). The engine gives the true quantile there because its band rule starts at 1E9. At 1.2E8
  and p = 1E-300, Excel is 119594626.4274513 and the engine and the math are ...483. Excel is 2.5E-14 off the
  math there: within R3's G, so Excel's value is the target.
- **GAMMA.INV band edges**: on both edges Excel is `#NUM!` at cells the engine puts inside the band. The old lower edge
  failed at 1E9 to 3.5E9, at offset - z of 37.6 to 38.8, below 39 (band-lower A1–A21); T-3350's edge, on main since,
  separates all 42. The upper edge fails at 3E9 to
  5E9 in the upper tail (band-upper A1–A13). The reverse also happens: Excel has a value where the engine is
  `#NUM!` at 7E9 to 9E9 in the lower tail (A26, A32, A33, A38, A39). So neither edge is a fixed constant in the
  engine's variables. The slope (1.35) or the variables themselves (offset, z) differ from Excel's search.
- **Closed form**: confirmed on 59 fresh cells. Excel's value is the Wilson–Hilferty start exactly where the density
  there is not a normal double, at 2.66E-4 off the math (the MATH-OVER-EXCEL case 3 of decision 0188 R3). Verdict (d)
  is about where the boundary lies and the type on either side. Inside the region decision 0188 R3 makes the math the
  target, not Excel's x0 (T-3364); this audit does not change that.

## Follow-up tasks

Filed with the cells:
- `HUGE_SHAPE_DENSITY_CUT` and `HUGE_SHAPE_CDF_CUT` (the wrong-type cells, Excel's real variable): T-3316, raised to P1.
- GAMMA.INV's huge-shape band (`HUGE_SHAPE_MIN`, the upper edge, the lower tail at 7E9 to 1.1E10): T-3409 and T-3396.
- The large-shape density at x/a 0.002 to 0.03: T-3617, which owns the same region of the T-3509 blind probe (filed on
  branch claude/hopeful-meitner-5oxf0g-blind-probe, not on main yet).
- The six bit-level constants and the closed-form cut's tag: T-3633.
