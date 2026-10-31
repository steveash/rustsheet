# Fit audit: thinly evidenced gamma-family FITTED constants (T-3530, WIP)

Decision 0188 R4 audit of the gamma-family `FITTED` constants with 6 or fewer evidence cells, on `main` 744815d06.
Evidence: probes-numeric `fit-audit-*` (excel-graph, 2026-10-31), the engine at 744815d06, and 50-digit mpmath
(regularised incomplete gamma; positive-term lower series in the far left tail at shapes of 1E6 and up).

**Status: interim.** The session ended before the bit-level constants were scored on both sides of their brackets.
Verdicts marked *provisional* or *not audited* are not final. No engine code changes on this branch yet.

Branches: T-3311 (PR #2599) moves `CLOSED_FORM_QUANTILE_MAX` and `CLOSED_FORM_STEP_MIN` to `closed_form.rs`
without changing them. T-3285 and T-3269 add new FITTED constants but change none of these.

Notation: "exact" means bit-exact, "≤15" means within 15 significant digits but not bit-exact, "off" means numeric and
beyond 15 digits, "err" means Excel's error differs from the engine's value (or the reverse).

| Constant (file, old tag) | Old evidence | New evidence (case, cells) | Verdict | Change |
|---|---|---|---|---|
| `HUGE_SHAPE_DENSITY_CUT` 6.9187E11 (gamma.rs:36, T-3271, 3 cells) | t2435 A26 `#NUM!`, A92 and A31 0 | fit-audit-t3271-density-cut, 50: 46 agree, 4 err | **(b) refuted** | Restriction or replacement needs a follow-up probe and task (see below) |
| `HUGE_SHAPE_CDF_CUT` 1.85E76 (gamma.rs:44, T-3271, 2 cells) | t2435 A100 0, A101 `#NUM!` | fit-audit-t3271-cdf-cut, 42: **pending** (oracle createSession failed) | not audited | none |
| `LOG_FORM_SHAPE_MAX` 3.0 (gamma.rs:67, T-2852, 6 cells) | t1049, gen-m9, m9-chisq-dist, m9-gamma-dist-a9 | fit-audit-t2852-shape-max, 52: 13 exact, 35 ≤15, 4 off (A25, A29 at 3 and 3.01; A34 3.2; A46 4.5) | provisional: not scored on both sides | none |
| `LOG_FORM_SHAPE_MIN` 0.5 (gamma.rs:84, T-2852, 2 cells) | gen-m9 A367, A386 | fit-audit-t2852-shape-min, 48: 12 exact, 36 ≤15, 0 off | provisional (a) candidate: no cell off on either side; bit-level split not scored | none |
| `HUGE_SHAPE_MIN` 1E9 (huge_shape.rs:17, T-3070, 1 cell) | t3070 A1 | fit-audit-t3070-shape-min, 39: 29 agree, 9 err, 1 off | **(b) refuted** | Follow-up task |
| `BAND_UPPER_EDGE` 27.25 (huge_shape.rs:74, T-3070, 2 cells) | t3070 A77, t2982 A45 | fit-audit-t3070-band-upper, 52: 37 agree, 10 err, 5 off | **(b) refuted** | Follow-up task |
| `BAND_LOWER_EDGE` 39 (huge_shape.rs:83, T-3070, 3 cells) | t2982 A44, t3070 A44, A370 | fit-audit-t3070-band-lower, 42: 24 agree, 12 err, 6 off | **(b) refuted** | Follow-up task |
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
  (x/beta = 1E-300) is 0. Every probed cell, these and t2435 A25, A26, A92 included, fits a rule on x / shape:
  `#NUM!` where x/shape is below a cut in (2.17E-309, 1E-308], 0 above it (UNVERIFIED: it needs a second probe at
  smaller shapes and x/shape between those values).
- **GAMMA.INV `HUGE_SHAPE_MIN`**: Excel is already `#NUM!` at shapes 6E8 to 9E8 for p = 1E-300 and 1E-250, and at
  9E8 for 1E-200 (A19–A33). The engine gives the true quantile there because its band rule starts at 1E9. At 1.2E8
  and p = 1E-300, Excel is 119594626.4274513 and the engine and the math are ...483. Excel is 2.5E-14 off the
  math there: within R3's G, so Excel's value is the target.
- **GAMMA.INV band edges**: on both edges Excel is `#NUM!` at cells the engine puts inside the band. The lower edge
  fails at 1E9 to 3.5E9, at offset - z of 37.6 to 38.8, below 39 (band-lower A1–A21). The upper edge fails at 3E9 to
  5E9 in the upper tail (band-upper A1–A13). The reverse also happens: Excel has a value where the engine is
  `#NUM!` at 7E9 to 9E9 in the lower tail (A26, A32, A33, A38). So neither edge is a fixed constant in the
  engine's variables. The slope (1.35) or the variables themselves (offset, z) differ from Excel's search.
- **Closed form**: confirmed on 59 fresh cells. Excel's value is the Wilson–Hilferty start exactly where the density
  there is not a normal double, at 2.66E-4 off the math (the MATH-OVER-EXCEL case 3 of decision 0188 R3).

## Follow-up tasks to file

None filed yet. The session ended first. The steps to file them are in T-3530's Handoff.
- Replace `HUGE_SHAPE_DENSITY_CUT` with Excel's real variable (x / shape), after a second probe.
- Re-fit GAMMA.INV's huge-shape band: `HUGE_SHAPE_MIN` and both edges, against t3070, t2982 and the fit-audit
  cells together.
- The large-shape density at x/a 0.002 to 0.03 (46 cells beyond 15 digits in fit-audit-t2638-ratio-min).
