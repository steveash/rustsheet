---
id: T-2933
title: "BETA.DIST at a shape below 1E-15: Excel has no fpser or apser, so the tiny-shape point runs plain bpser or w1 bpser (probes-numeric t2464-r2-bpser-small-tiny, -tiny2: 194 visibly-off cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
BETA.DIST's and BETADIST's cdf with one shape below 1E-15 gives Excel's value to 15 digits. bratio's dispatch then
skips TOMS 708's fpser and apser, as Excel does: an unswapped point (x <= 0.5) runs the ported bpser(a0, b0, x0), and
a swapped point with x0 >= 0.3 runs the ported w1 = bpser(b0, a0, y0). Today both go back to the Cephes kernel.

## Context
Gap triage 2026-10-27 (eval on origin/main 43ac55b). No open task names these two probes. T-2505's Context said "no
fpser; apser falls through the dispatch", but its Outcome did not ship that, and T-2847, which took T-2505's other
criteria, does not cover it.
- Code: `small_shape_branch` in crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs returns
  `Branch::Fpser` for `b0 < EPS.min(EPS * a0)` and `Branch::Apser` for `a0 < EPS.min(EPS * b0) && b0 * x0 <= 1`.
  `ported_tails` has no arm for either, so `beta_dist_lower_tail` falls back to `beta_reg_complement` (Cephes).
- Excel's rule: docs/excel-semantics/statistics.md "Incomplete beta bpser: a forward series times EXP(fl(a LN x) -
  lnB) (T-2464)", bullet "No fpser". The research is in docs/research/t2464/r2-bpser-small/RESULTS.md items 4 and 5
  (`tiny_alt.py`, `tiny_noapser.py`, `lib.py` `V(nofpser=True, noapser=True, lg="GL53/glalgdiv")`). Its counts, as
  exact/4 ulp/15 digits:
  - for b0 < 1e-15·min(1, a0), plain bpser gives 132/184/184 of 184 and TOMS fpser 6/32/95;
  - for a0 < 1e-15 (the swapped cells), falling through bratio's dispatch gives 14 of 15 within 15 digits where
    x0 >= 0.3 (w1 bpser), against TOMS apser's 6.
- Excel is the less accurate one here, so this is the `visible` stage, not `accuracy`. An mpmath check at 50 digits
  (`betainc(a, b, 0, x, regularized=True)`, triage scratch script) on the 223 visibly-off cells of -tiny puts Excel 2E-15
  to 4E-15 relative off the true value on most cells. The engine is closer on 152 of them. Examples:
  - `BETA.DIST(0.02432,1.53,4.21e-16,TRUE)` (A100): Excel 9.474745704246278e-19 (3.4E-15 off the truth), ours
    9.474745704246253e-19 (7.9E-16 off).
  - `BETA.DIST(0.3402,1.53,4.21e-16,TRUE)` (A101): Excel 6.736848228426744e-17, ours 6.736848228426725e-17.
  - `BETA.DIST(0.6132,0.3433,2.29e-18,TRUE)` (A110, swapped, x0 = 0.3868): Excel 7.097205776528748e-18, ours
    7.097205776528769e-18.
  RESULTS.md item 4 traces the ~3E-15 offsets to ln B, where GAMMALN of the tiny shape is about 40. They are not a
  misparse of the literals: probe tiny2 repeats every row with exact-piece arguments and gives the same values.
- probes-numeric is ungated (no baseline file), so the counts below come from `make eval` (report.json `cases[].cells`)
  or `make eval-show CASE=probes-numeric/t2464-r2-bpser-small-tiny.case.json`.

Owned cells (excel-graph; visibly off on 2026-10-27, 97 in each case, the same cell names in both):
- probes-numeric/t2464-r2-bpser-small-tiny.case.json and probes-numeric/t2464-r2-bpser-small-tiny2.case.json (97 each):
  Calc!A19, A20, A33, A35, A36, A37, A38, A39, A49, A55, A66, A67, A69, A70, A71, A81, A82, A84, A85, A86, A88, A97, A100,
  A101, A102, A103, A104, A110, A113, A115, A117, A118, A119, A120, A128, A133, A134, A135, A136, A145, A146, A149, A150,
  A152, A161, A163, A165, A166, A167, A168, A187, A194, A196, A200, A205, A210, A213, A216, A227, A228, A229, A230, A231,
  A241, A243, A244, A245, A246, A247, A248, A251, A257, A258, A260, A261, A262, A264, A291, A292, A293, A305, A308, A309,
  A312, A321, A328, A338, A340, A342, A344, A353, A354, A356, A357, A358, A360, A382.
  Of these, 91 per case are x <= 0.5 (fpser today) and 6 are swapped with x0 = 1 - x >= 0.3 (apser today).

## Acceptance criteria
- [ ] `small_shape_branch` no longer returns `Fpser` or `Apser` for BETA.DIST's cdf. A tiny b0 takes the bpser arms
      that follow in the 1992 order, and a tiny a0 falls through to `W1Bpser` or `BupBgrat`, as the model's
      `nofpser`/`noapser` dispatch does. If F, BINOM or NEGBINOM already call the port when this lands (T-2847), they
      take the same dispatch. Unit tests pin the three example cells above to Excel's bits, or to within 15 digits
      where the model is not bit-exact.
- [ ] On each of the two cases, the 91 x <= 0.5 owned cells are within 15 digits (the model: 184 of 184 such cells,
      132 bit-exact; report the engine's exact/4 ulp/15-digit counts in the Outcome). At least 5 of the 6 swapped
      x0 >= 0.3 owned cells are within 15 digits (model: 14 of 15).
- [ ] No other probes-numeric case gains a visibly-off cell (per-case counts against the same run on `main`; list
      any bit-exact cell lost, decision 0083). Gated suites: no new baseline line (`make eval-baseline` refuses
      regressions).
- [ ] statistics.md's bpser entry gets an "Implemented in" line for the tiny-shape dispatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 126 swapped cells per case with x0 < 0.3: bup + bgrat. Their visibly-off values need bgrat (T-2510), and
  Excel's 1E-14 to 3.8E-13 offsets at 0.04 <= y < 0.25 are unsolved (T-2512 item 6). They stay on today's kernel.
- The b < 1 bpser bit-exact ceiling (T-2512 item 4) and GAMMALN's last bits at the shape arguments (T-2122).
- Moving F, BINOM and NEGBINOM onto the port (T-2847).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value (an expected 0 must be exactly 0). If bit-exact criteria are still unmet then, they move to a follow-up
task with no campaign key and `defer: phase-1b`, named in the Outcome. Regressions that stay within 4 ulp follow
decision 0083's allowance.

Visibly-off cells on the 2026-10-27 eval run (origin/main 43ac55b) that this task owns: 194, all BETA.DIST cdf with
b between 1E-18 and 6E-16 (the two cases above, 97 each). Excel is not correctly rounded on these cells (see
Context), so 15 digits means reproducing Excel's bpser at the tiny shape, not a more accurate kernel.
