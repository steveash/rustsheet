---
id: T-3457
title: "BETA.INV at a shape of 1: the closed form where our crossing search lands in BETA.DIST's flat w1 band (m9-beta-inv A69, A149)"
milestone: M9
lane: functions-stat
status: blocked
priority: P1
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.INV and BETAINV with a shape of exactly 1 (alpha = 1 or beta = 1) and p of 1E-15 or more return a value
within half a unit of the 15th significant digit of Excel's, as the shape-1 closed form does at all 127 literal
Excel points of that kind in eval/. The four visibly-off cells below pass, and none of the shape-1 cells that pass
today moves off.

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main, then `scripts/exit_owners.py --milestone M9` and
  `--all-off`): these 4 `rate:BETA.INV` cells have no open owner. T-2511 (done) recorded them as moved by T-3007
  without filing an owner; T-3398 (its remainder) names its m9-beta-inv cells A31, A102, A103 and hands 12 to
  T-3388 and 5 to T-3351, and these are none of them.
  - probes-numeric/m9-beta-inv Calc!A69 `BETA.INV(0.999,1,100)`: Excel 0.06674569920300899, ours
    0.06674569920300838 (40 ulp, 6.1E-16 absolute; half a unit of the 15th digit is 5E-17)
  - probes-numeric/m9-beta-inv Calc!A149 `BETA.INV(0.001,100,1)`: Excel 0.933254300796991, ours 0.9332543007969905
  - probes-numeric/t1745-beta-inv-roundtrip Calc!J67 and Calc!J144: the same two points (row 67 = A69's, row 144 =
    A149's), same values.
- Root cause (T-2511 context, statistics.md "Incomplete beta at a shape of 1 outside the closed forms (T-3007)"):
  at a0 = 1 with y0 > 0.9, BETA.DIST's cdf forms the small tail as w1 = (0.5 - w) + 0.5, so it moves in steps of
  one ulp of w (about 1E-16 absolute, 1E-13 relative at a tail of 0.001). `beta.rs::beta_quantile` refines to the
  crossing of that cdf (`refine_increasing` on `beta_reg_upper`/`beta_reg_complement`), and the crossing is a flat
  band tens of doubles wide; the search stops anywhere in it. T-3007 moved these cells from -4 and +1 ulp to -40 and
  -5 ulp while making Excel's BETA.DIST at those x bit-exact, so the forward cdf is right and the inverse must not
  solve it there.
- Accuracy, not a fit: a decimal reference at 50 digits gives the root 0.066745699203008956 for A69 (Excel +5.1E-16
  relative, ours -8.6E-15) and 0.93325430079699104 for A149 (Excel -3.6E-17, ours -6.3E-16). Excel is close to
  correctly rounded and we are not.
- Evidence for the closed form: over every literal `BETA.INV`/`BETAINV(p,a,b)` cell in eval/ with a = 1 or b = 1 and
  1E-15 <= p < 1 (127 cells: m9-beta-inv 63, t1745-beta-inv-roundtrip 62, probes/t2435-dist-extremes 2), Excel's
  value against x = p (a = b = 1), x = -expm1(log1p(-p)/b) (a = 1) or x = exp(ln(p)/a) (b = 1), all in double with
  libm: 80 bit-exact, 34 at 1 ulp, 5 at 2, 6 at 3, 2 at 12 (m9-beta-inv A78 `BETA.INV(1E-10,2.5,1)` and its t1745
  row), and every one within 15 digits. `1 - (1-p)^(1/b)` with POWER is worse at small p (A51 `BETA.INV(0.001,1,2.5)`
  640 ulp), so do not use it.
- Below p = 1E-15 Excel leaves the closed form: probes-numeric/t1543-inverse-cdf-tiny-p A73:A75
  (`BETA.INV(1E-300|1E-200|1E-100,1,1)`, Excel 1.0000000000000237E-300 etc.) and A86 (`BETA.INV(1E-200,1000,1)` =
  0.2064, the true quantile 0.63). Keep today's path there (the existing `BETA_QUANTILE_STATRS_P_MIN` = 1E-15 cut);
  do not add a new cut.
- Side effects to expect and record (not this task's cells, so do not claim them): the closed form also puts T-3388's
  m9-beta-inv A8 `BETA.INV(1E-10,0.5,1)` and A43 `BETA.INV(1E-10,1,1)` (and t1745 J7, J41) within 15 digits, and
  T-3351's A42 `BETA.INV(0.9999999999,1,0.5)` at Excel's 1.0 (ours `#NUM!`). Name them in the Outcome so T-3388 and
  T-3351 drop them.
- Quirk entry: docs/excel-semantics/statistics.md "BETA.INV at a shape of 1 is the closed form to 15 digits for p of
  1E-15 or more (T-3457)".
- Code: crates/rustsheet-engine/src/functions/statistical/beta.rs `beta_quantile` (BETA.INV, BETAINV via
  `beta_inv_of`); search for an existing log1p/expm1 helper before writing one (directive 3); a shape above 1E5 takes
  `beta_quantile_crossing` today, keep that order or show the closed form is no worse there on the t1538 probes.

## Visible-parity stage
Decisions 0083 and 0114 (`accuracy`): Excel is within a few ulps of the correctly rounded quantile at these cells and
the engine is not, so the fix is the accurate closed form, never a fit to Excel's bits. Done when every cell this
task owns is within half a unit of the 15th significant digit of Excel's value. Bit-level residue (the 45 shape-1
cells 1 to 12 ulp from Excel) goes to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] probes-numeric/m9-beta-inv Calc!A69, Calc!A149 and probes-numeric/t1745-beta-inv-roundtrip Calc!J67, Calc!J144
      are within half a unit of the 15th significant digit of Excel's value (`make eval-show`).
- [ ] All 127 shape-1 cells listed in Context with 1E-15 <= p < 1 are within 15 digits of Excel; the Outcome gives
      the bit-exact count before and after.
- [ ] No new known-failure line in any eval/baseline file, ironcalc-calc BETADIST_BETAINV.xlsx included
      (`make eval-baseline` refuses regressions); BETA.INV visibly-off cells only shrink in report.md.
- [ ] Unit tests in beta.rs pin A69, A149 and one p < 1E-15 point that keeps today's search.
- [ ] statistics.md's T-3457 entry gets `Implemented in:`; the cells of T-3388 and T-3351 this fixes are named in the
      Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes other than exactly 1 (T-3388, T-3398); p near 1 at other shapes (T-3351); p below 1E-15 (T-2808, T-1871).
- BETA.DIST's forward cdf at a shape of 1 (T-3192, T-3007's open items).

## Blocked because
Blocked because: the baseline step needs `rustsheet-eval update-baseline --allow-regressions`, which this session's
permission policy refused; the owner (or a session allowed to run it) has to run it.

The code, unit test and semantics entry are on this branch. With it, `make eval-check` reports one gated regression:
gen-m9/589f42e67f4b4b8d Calc!A221 `BETAINV(0.001,1,42)` goes from bit-exact to 1 ulp off (Excel
2.3821152785727803E-5, ours 2.3821152785727806E-5, within 15 digits). It also reports one fixed cell each in gen-m9,
ironcalc-calc and probes. The trade meets all four conditions of decision 0083's standing ulp-regression allowance,
so the step needs no decision record of its own:
1. the one newly failing gated cell is numeric and 1 ulp off; every newly failing ungated cell is 1 ulp off;
2. no cell becomes visibly off: all 70 changed cells are within 15 digits of Excel;
3. visibly-off cells fixed: m9-beta-inv A69, A8, A43 and their t1745 rows J67, J7, J41; t2435 A72 (36,740 ulp, now
   2) and A112 (256 ulp, now 0);
4. the regressed cells are listed below, and follow-up T-3460 (`defer: phase-1b`) names them.

Next: run `scripts/eval_guard.sh target/release/rustsheet-eval update-baseline --allow-regressions`, then
`target/release/rustsheet-eval evidence-floor --update`. Check that the baseline diff adds only the A221 line to
eval/baseline/gen-m9.txt, then run `make check` and the review panel. The excel-fidelity reviewer quotes A221 and its
1 ulp.

## Outcome (draft, before the baseline step)
- `beta.rs::beta_quantile_shape_one`: at a shape of exactly 1 and p >= 1E-15, BETA.INV and BETAINV return the closed
  form (p; -expm1(log1p(-p)/beta); exp(ln(p)/alpha)) instead of the crossing of BETA.DIST's cdf, which is flat there.
  Shapes above 1E5 still take `beta_quantile_crossing` first, and p below 1E-15 keeps the search (t1543 A73:A75).
- Eval, every BETA.INV/BETAINV cell, against the run on 1af9bd6c: 32 cells become bit-exact and 18 move from
  bit-exact to 1 ulp. Twenty more change while still failing. The bit-exact count among the changed cells goes from
  18 to 31 of 70, and all 70 are within 15 digits.
  - The task's own cells: m9-beta-inv A149 and t1745 J144 are bit-exact. A69 and t1745 J67 go from 44 ulp to 3, so
    they are within 15 digits.
  - Gated suites: gen-m9 +1 fixed (2a18b2f5a4dc43ac A44), 1 regressed (589f42e67f4b4b8d A221); ironcalc-calc
    BETADIST_BETAINV.xlsx H72 fixed; probes t2435-dist-extremes A112 fixed and A72 from 36,740 ulp to 2.
  - Regressed to 1 ulp, all ungated (probes-numeric): m9-beta-inv A12, A13, A40, A41, A55, A57, A83, A117; t1745 J11,
    J12, J39, J40, J53, J55, J80, J113; t1543 A90.
  - Cells of other tasks this fixes (T-3388 and T-3351 should drop them, not claim them): m9-beta-inv A8 and A43,
    t1745 J7 and J41 (T-3388). T-3351's A42 `BETA.INV(0.9999999999,1,0.5)` stays `#NUM!`: the closed form rounds to
    1, which `beta_inv_of` refuses; T-3351's own rule decides that cell.
- Not a fit: the closed form was fixed by the task before scoring, and no selector was chosen from the scores. A
  "keep the crossing within 1 ulp of the closed form" selector would have kept A221, but it is a keep-matched branch
  (decision 0188 R4), so it was rejected.
