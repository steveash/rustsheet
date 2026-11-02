---
id: T-3818
title: "BETA.DIST density at one large shape on the T-3568 bup-terms probes: 7,026 unowned visibly-off cells (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
`BETA.DIST(x,a,b,FALSE)` with one small shape (a from 0.5 to 40) against one large shape (b from about 20 to 2,900),
the densities of the T-3568 bup-terms probes, matches Excel's value to 15 digits on the tune and hold cases, or each
cell left over is named with its reason and an owner.

## Context
- Gap triage 2026-11-02 (report of `make eval` on 2026-11-02, excel-graph; `python3 scripts/exit_owners.py --all-off`):
  these are the only large unowned visibly-off group. T-3568 (done, research) committed the probes to test its "loop of
  one-term bups" hypothesis (docs/research/t2464/r5-bup/RESULTS.md, `mkterms.py`, `terms.py`); its engine follow-up
  T-3712 lists t3568-r5-bup-tune, -hold and -fresh, not these two. Every cell is a density
  `BETA.DIST(y0, c + j, a0, FALSE)` for j < n at a bup+bgrat point.
- Counts (2026-11-02): terms-tune 5,898 cells, 275 bit-exact, 3,602 visibly off; terms-hold 5,967 cells, 283 bit-exact,
  3,424 visibly off. The rest are NumericUlp/NumericClose within 15 digits.
- Examples (Excel vs ours, terms-hold): Calc!O231 `BETA.DIST(0.258,14.561,2283.22,FALSE)` 5.537148910195256e-266 vs
  5.537148910194687e-266; Calc!M264 `BETA.DIST(0.0782029528482,13.0,1617.6,FALSE)` 4.0179058036923066e-38 vs
  4.017905803692237e-38; Calc!AA20 7.975405748028135 vs 7.975405748028151 (`(1669435*2^-25+30215771*2^-51)`,
  shape `(28101836*2^-20+53687091*2^-46)`, 720.3).
- Stage `visible`, not `accuracy`: on 9 sampled cells against 50-digit mpmath, Excel is 2.7e-15 to 1.8e-13 relative
  off the density and the engine 1.4e-15 to 6.7e-14; Excel is the less accurate one. Both are within Gate A's 1e-9,
  so decision 0188 R1(c) counts these as Phase 1b evidence, and R5 parks the work: hence `defer: phase-1b`.
- Suspected root cause: the same family as T-3250 and T-3244 (Loader's saddle point at a large exponent: the error
  grows with |ln f|, here up to |ln f| of about 660) for min(a,b) > 1, and the min(a,b) <= 1 path's lnB (algdiv
  for b >= 8) for the a = c < 1 columns; T-3569 found no x87 storage variant of algdiv that fits on the web pins,
  and T-3750 (M14, blocked) asks desktop cells for them. Start from whichever of T-3220, T-3244 or T-3250 lands first;
  do not fit a second density model beside theirs (directive 3).
- Fit lab: these two cases are already a tune/hold pair (fit on terms-tune, score once on terms-hold). Decision 0188
  R4 binds any fit here.

Cells (gap triage 2026-11-02): every cell of these cases:
- probes-numeric/t3568-r5-bup-terms-tune.case.json (fit)
- probes-numeric/t3568-r5-bup-terms-hold.case.json (hold-out: score once, never fit)

## Acceptance criteria
- [ ] Every cell of both cases is within 15 digits of Excel's value, or each cell left over is named here with its
      reason and an owner; the per-case counts (exact, within 4 ulp, within 15 digits) before and after are in the
      Outcome.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7): hold-out counts on terms-hold reported beside the
      tune counts and the correctly rounded candidate's; a fresh log-uniform sweep over the same shape range filled
      after the model is frozen, with its result in the Outcome; the Gate A envelope result on that sweep; every fitted
      constant, cut or table tagged `FITTED(T-3818, <n> cells, <range>)` and never applied outside its range; no fit
      below the oracle's reproducibility floor (decision 0112); no branch that only keeps cells that already matched.
- [ ] No new visibly-off cell in any suite: the cells of T-3220, T-3244, T-3250 and the min(a,b) <= 1 tasks keep their
      verdicts, and the known-failure lines in eval/baseline only shrink.
- [ ] statistics.md's beta density entries record what was found with its hold-out score, or the negatives tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf on the same points (T-3712) and the bup terms themselves (T-3568's RESULTS.md).
- Both shapes large at n < 300 (T-3244), the a = 1025 column (T-3250), the residue at n < 90 (T-3220).

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value. Bit-level
residue goes to a follow-up with `defer: phase-1b`.
