---
id: T-3395
title: "FITTED weak counts and untagged cuts: t_dist p cuts, f_density LARGE_DF_MIN and LOG1P_SERIES_LIMIT, RATE secant and IRR model"
milestone: M0
lane: quality
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Every `FITTED` tag's cell count and range describe the cells that actually sit near its cut, and the cuts the T-3361
review flagged are either tagged or documented as not fits (decision 0188 R4.3).

## Context
Left over from T-3385, which settled the five `range unknown` tags and `EXPM1_FORM_LIMIT` (method: temporary
`eprintln!` at the branch, a full `rustsheet-eval run`, the logged inputs reduced to ranges; revert before committing).
Still open from the T-3385 task context (crates/rustsheet-engine/src/functions/):
- statistical/t_dist.rs `T_TAIL_P`, `T_STATRS_MIN_P`: the cited probe t1531-t-tiny-p covers p 1E-300..1E-10, which does
  not reach the cuts at 1E-5 and 1E-8; find the cells either side or say none bracket them. `HUGE_SHAPE_MIN` counts only
  the 1E9 side; `T_CENTRE_C` lists only the c = 0.1 cells.
- statistical/f_density.rs `LARGE_DF_MIN` (16, r5-f-bigconst `BIG_MIN`): fitted cut or probe-design edge (r5-f-bigconst
  section 4.5)? Tag it if fitted. Trace the 5752 / 4232 counts on the Stirling constants to a named file
  (docs/research/t2464/r4-f-small-df gives both max(d) <= 10 and max(d) <= 30). `LOG1P_SERIES_LIMIT` (0.375): tag or
  record why it is not a fit.
- T-3385 review (excel-fidelity, MINOR): `EXPM1_FORM_LIMIT`'s bracket 0.99505..1.0375 has no discriminating cell inside it
  (t2178's cells there fit both forms); a probe of inputs where the forms disagree would pin the cut. The T-3385 ranges came from a reverted `eprintln!` run: a committed range script (or a
  fit-lab listing) would make them reproducible.
- financial/financial_util.rs RATE secant tolerances and the IRR model (T-2005, T-1525, T-1533): tag or record why not.

## Acceptance criteria
- [x] Each item above is tagged with a range read from the cells, or its doc comment says why it is not a fit;
      quality/fitted.json updated with `--update` if the count changes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any fitted value (T-3364, T-3365).

## Outcome
No value or behavior change: doc comments, tags, two probe cases, semantics notes, decision 0195.
Method for the t_dist and huge-shape cuts: a temporary env override of each cut (reverted, never committed), a
`rustsheet-eval run` of probes-numeric, probes, gen-m9 and ironcalc-calc per setting, and every cell whose value
changed scored by its ulp distance to Excel against an unchanged run; a temporary `eprintln!` run counted the inputs
near each cut.
- t_dist.rs, re-measured after merging main's T-2511 (which sends every df from 2 to the rounded-y crossing):
  - `T_TAIL_P` (1E-5): it moves 25 cells, df 1 and 2, q 5E-11..5E-3. Lower cuts are worse (1E-11: 15 worse, 4
    better), 1E-3 brings one df-1 cell closer, and 1E-2 is worse on net, so the cells place it in 1E-5..1E-3.
  - `T_STATRS_MIN_P` (1E-8): no cell moves at any cut from 1E-20 to 1E-7; it is a statrs-failure guard. The tag
    stays, and says so.
  - `T_CENTRE_C` (0.075): bracketed 0.05..0.1 by gen-m9's six c = 0.05 cells (a cut of 0.04 breaks them) and the
    c = 0.1 cells; two failing t3349 cells at c 0.0787 and 0.0985 favour 0.0985..0.1 (13 cells move).
  - Before T-2511 the same runs had favoured 5E-4..5E-3 for `T_TAIL_P` and statrs over bisection down to 1E-20; the
    rewrite moved those cells to the new path, so the earlier numbers no longer hold.
- huge_shape.rs `HUGE_SHAPE_MIN` (1E9): a cut of 2E9 breaks t3070 A1 and A335 (shape 1E9); a cut of 1E8 changes no
  cell (13 calls at shapes 1E8..7E8 give the same value from either path), so nothing pins it from below.
- f_density.rs: `LARGE_DF_MIN` is the r5-f-bigconst probe-design edge (its section 4.5 scan, after the freeze and
  without a hold-out, prefers 8); `LOG1P_SERIES_LIMIT` is TOMS 708 brcomp's 0.375 (Excel's ln(1 - x) switch to the
  bit), confirmed for r > 0 by r4-f-brcomp's hold4 slopes. Neither is tagged; both comments say why. The 5752 cells
  were all `s4_kscan.out` cells plus the d2 <= 10 `s4_kscan_tdens.out` cells; the fit itself (`s19_sidefit.py`
  MINC 25, 56 groups) used 4,946 cells, df 1..10 and d1 1 up to d2 30, now the tag. 4,232 is r3-loader-const
  `a10_grid.py`'s 529 grid shapes x 8 x of t2513-r3-loader-const-grid1.
- hyperbolic.rs `EXPM1_FORM_LIMIT`: new probes-numeric t3395-hyperbolic-form-switch (120 cells) and -2 (94) at
  points where the engine's two forms differ narrow Excel's switch from 0.99505..1.0375 to 1 - 5 ulp..1 + 2 ulp,
  inside which the forms agree, so 1 fits every cell (numbers.md). Both cases pass. They are excel-graph only and
  their cells differ by about 1 ulp, inside the web-against-desktop floor, so the narrowing is web-only so far.
- financial_util.rs, tagged (decision 0195, fitted lock 81 -> 86 by `--update`): RATE's start offset and residual
  stop (782 cells of the fit lab's `rate` set), RATE's cap tolerance (178 cap-probe cells, one cut inside all three brackets,
  9.4E-5..1.02E-4), IRR's start offset and residual stop (263 cells of the `irr` set) and `IRR_FAR_ROOT_BOUND` (35
  probe cells, 9999999.994..1E+7). These and f_density's small-shape constant apply past their tagged ranges:
  T-3413. Not fits, now said in the comments: RATE's and IRR's 1E-7 step stops (Microsoft's documented bounds), `RATE_MAX_STEPS` and `IRR_GUESS_BOUND` (pinned exactly), the loop caps (directive 5).
- R4 guards: no value changed, so no hold-out or fresh sweep is owed; the in-sample scores above are why T-3410 owns
  moving the t cuts under R4.
- Not done: a committed range script (the T-3385 review's optional point); the cut-moving method needs an override
  hook the engine must not carry, so it stays a recipe here.
Follow-ups: T-3410 (move `T_CENTRE_C` and `T_TAIL_P` under R4), T-3413 (fits applied past their tagged ranges). Eval delta: +214 cells (the two new probe cases, all bit-exact); no
other change.
