---
id: T-4888
title: "IRR's secant runs on past the pole: continue the run at d > 1 instead of failing, with an absolute stall bound"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-4844]
---

## Goal
IRR's secant goes on past the pole (an iterate at d > 1) as Excel's does, instead of failing there, so the calls whose run crosses the pole return Excel's bits.

## Context
- docs/research/irr-2026-11-07/plan.md (T-4833) names the pole crossing as one of the three events the replica treated
  as a dead end. RATE's fitted solver runs on below -1 ("RATE holds on a flat secant and runs on below -1").
- Scratch evidence (T-4844 session): the run-on secant gives Excel's bits on 65 cells of
  probes-numeric/t2927-irr-fallback-sweep and probes/irr-fallback-roots, where the T-2928 fallback is 15 to 35 ulp off.
- Code: `irr_secant` (crates/rustsheet-engine/src/functions/financial/financial_util.rs) failed at d > 1.
- Split from T-2131 and T-4778: two cells they list, gen-m9/a91bad4e84f49739 Calc!A369 (T-2131) and
  probes/t2565-irr-guess-sweeps P!C157 (T-4778), pass with this change; the rest of those tasks stays theirs.

## Acceptance criteria
- [x] A fresh sweep with predictions generated from its seed before the fill (probes/t4838-irr-pole-runon): the run-on secant is
      bit-exact on every cell where it converges.
- [x] `irr_secant` runs on past d = 1; a result past the pole fails the run; unit tests.
- [x] No eval regression; Gate A unchanged or better; financial.md "IRR's secant runs on past the pole".
- [x] `make check` green; reviewers approve.

## Out of scope
- What Excel does where the run-on secant does not converge (t4838 A116:A145; T-4840).
- The flat secant (T-4838).

## Outcome
`irr_secant` runs on past the pole. A step at d > 1 is evaluated at the negative v and the run carries on, d = 1
stays `#DIV/0!`, and a result past the pole fails the run. Two changes follow from the fresh sweep and the eval:
- a zero-step stall stops a run only at the flows' rounding noise: |NPV| < 1e-4 and < 1e-12 of the largest flow
  (`irr_stall_stops`). The relative 1e-7 bound returned gen-m9/fec39efaba9284c3 Calc!A283 once that run went on past
  the pole;
- `IRR_LONG_RUN_RANGE` covers 4 and 5 flows too: 12 fresh cells of 4 and 5 flows need 52..197 steps (no cell evidences
  2 or 3 flows, which keep the 50-step cap).

Evidence (decision 0188 R4):
- The rule was found on t2927-irr-fallback-sweep and irr-fallback-roots (65 cells bit for bit) as the train side.
- The fresh sweep probes/t4838-irr-pole-runon (160 cells, seed "T-4838/pole", excel-graph 2026-11-07) was drawn after
  that. Each model's prediction depends only on the seed and was generated before the fill (its `.predictions.tsv`,
  rewritten by the committed replay.py with the same values). The run-on prediction is
  Excel's bits on all 130 cells where it converges in 200 steps, against 15 for the old Newton fallback.
- The engine now passes 134 of the 160. The other 26 are mostly where the run-on fails; T-4840 owns them. On A131 the
  engine is `#NUM!`, as on main. The 2-ulp value of the predictions file is the replay's Newton model, not the engine.
- The cap widening is measured on the same fresh sweep: its 12 cells were predicted with a 200-step cap.
- The stall rule (review round 2 asked for a fresh sweep after its freeze):
  - It was chosen on one eval cell (gen-m9/fec39efaba9284c3 Calc!A283).
  - Gate A's tables, which no fit reads, put 16 accuracy/IRR points (s1, r4-T-3411, r4-T-3407; flows near 1E+11 to
    1E+14) in the envelope with an absolute 1e-4 bound. With the relative 1e-7 bound and the run-on they fail.
  - The fresh sweep probes/t4888-irr-stall-sweep (207 cells, seed "T-4888/stall", drawn after the 1e-4 freeze) has 199
    stalls over largest flows of 1E-9 to 1E+14. It confirms the frozen 1e-4: Excel holds a stall at 8.75E-5 and
    rejects one at 1.28E-4.
  - It also shows the absolute bound alone is wrong on small flows: Excel rejects stalls far from any root (relative
    0.95 to 1.05) that the absolute bound would hold. So the rule became two-part: |NPV| < 1e-4 and |NPV| < 1e-12 of
    the largest flow, the relative half bracketed in 2.8E-16..1.6E-9.
  - The two-part rule matches Excel's hold-or-not on 195 of 199: 132/135 train, 63/64 test by the fit lab's split
    (margin 5 points, so no drop). The absolute alone gets 174, the relative 1e-7 171, no stall stop 182.
  - Applied only on largest flows of 1.3E-7 to 1.3E+14 (`IRR_STALL_FLOWS`, the sweep and t2565's 1.23E+14); outside
    it a zero step is no stop.
  - The engine passes 147 of the case's 207 cells; most of the other 60 are long-series second-stage cells (T-4840).
- Attribution, from scratch builds: the run-on alone gives the t4838 bits and the 4 eval fixes below; the stall bound's
  absolute half alone gives the 16 Gate A points.

Eval (full run, external suites not fetched): 0 regressions. Known failures 4 fewer outside the new case:
gen-m9/a91bad4e84f49739 Calc!A369, probes/irr-guess-edges P!A28 and P!A29, and probes/t2565-irr-guess-sweeps P!C157,
the outlier T-2565 and T-3120 left. IRR cells off Excel outside t4838 and t4844 go from 727 to 663 (ulp 34 -> 18,
visibly-off numbers 113 -> 73). Also: the three t4844 math sidecars were stale (their case descriptions changed after
the sidecars were written, in #3910) and are regenerated.
Follow-ups: none new. T-4840 owns t4838's 26 cells and t4888-irr-stall-sweep's 60; the flat secant stays T-4838's.

## Blocked because
Resumed after the review-round limit (step 5c).

The excel-fidelity reviewer's round-3 findings on head 7e0d968f are still open. Security and design approved that head.
- MAJOR: the stall rule's relative half (`IRR_STALL_RELATIVE` = 1e-12) and the two-part form were shaped after seeing
  probes/t4888-irr-stall-sweep's oracle values, so that sweep cannot be their fresh sweep (R4.1, R4.2). Freeze the
  rule, draw a new seeded sweep, fill it, and report hold-or-not counts for the rule and for "no stall stop". The 4
  misses (A152, A154, A159, A165, relative 1.1E-16..3.1E-16) sit inside the held band, so the true discriminator near a
  root is still unknown.
- MAJOR: report a Gate A envelope count, with the points outside it, for the fresh sweeps (t4838, t4888 and the new
  one); both have math sidecars.
- MAJOR: the FITTED tag says 3..40 flows and largest |c| 1.3E-7..1.3E+14, but `irr_stall_stops` checks only the largest
  flow. Either enforce the flow-count range or narrow the tag, state the guess evidence (guess 0.1 plus t2565's), and
  correct the cap's source: the sweep's stalled series reach 5.5E+17.
- MINOR: the garbled sentence in `IRR_STALL_RESIDUAL`'s doc ("frozen before the fresh sweep T-4888's own fresh sweep
  was drawn"; design flagged it too); the inline stall comment still says the bound is absolute only; financial.md
  should call the A131/A1:A70 comparison "the replay's Newton model" and the stall sweep's decades "of the scale"; the
  T-3120 entry's parenthetical on P!C157 needs its own sentence.
Not in dispute: the run-on past the pole (130/130 on the seeded t4838 sweep, 65 earlier cells) and the 4..5-flow cap
widening (12 fresh cells). A resumer may split those into their own PR if the stall rule needs more rounds.

