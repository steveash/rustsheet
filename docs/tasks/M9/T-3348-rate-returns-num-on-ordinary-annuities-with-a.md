---
id: T-3348
title: "RATE returns #NUM! on ordinary annuities with a nonzero fv"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
RATE returns Excel's rate for ordinary annuities with fv ≠ 0 where the engine returns `#NUM!` today, for example
`RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)`, which is 0.0431054691928255 in Excel. Together with
T-3347 (IRR) this is the first task of the accuracy stage.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 2). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_rate` in crates/rustsheet-engine/src/functions/financial/annuity.rs and its iteration (start, step,
  stop rule, the 20-iteration limit Excel documents). Find why the search fails to converge or leaves the domain on
  this input; check the guess-free start against Excel's.
- Prior evidence: T-2005 (done) fitted RATE's solver with per-iterate traces, and T-0916 and T-1525 (done) probed
  Excel's iteration and termination (start, step, stop rule). That behaviour is evidence-based: change it only against
  the new oracle case. The 2026-10-31 assessment found Excel returning a value on these inputs, so a stop rule that
  gives `#NUM!` here is not Excel's.
- Excel's RATE with 1 or 2 periods is about 1e-2 off the math at rates near 1e-8 (decision 0188, "Where Excel itself is
  far from the math"). Decision 0188 R3 lists no RATE case, so Excel's value stays the target there; it is out of scope.
- `annuity.rs` `DISCOUNTED_INTEREST_FROM` is a tuned cut-point of the code audit; do not move it in this task (T-3365).

## Acceptance criteria
Re-scoped in review round 1 by decision 0192 to what this task delivers. The original criteria it cannot meet are
moved, each with its note, below the list.
- [x] `RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)` is 0.0431054691928255 (a unit test and a probe cell).
- [x] The `#NUM!` class is fixed. Every probe cell where the engine gave `#NUM!` on a flat secant and Excel a number
      now has Excel's type: probes/t3348-rate-fv-nonzero goes from 126 to 186 of 186, all 60 misses fixed.
- [x] The run below -1: an iterate at or below -1 runs on, and only the result must be above -1.
      probes/t3348-rate-below-minus-one goes from 20/50 to 50/50 bit-exact.
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states). The baselines only gain lines.
- [x] Reviewer check 7 guards for a structural model, kind (c), recorded in the Outcome:
      - the hold-out split, fixed before scoring, where the model beats the correctly rounded candidate;
      - a fresh probes-numeric sweep drawn and oracle-filled after the freeze, with its Gate A envelope and every
        point outside it listed.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [x] `make check` green.
- [ ] Reviewers approve, decision 0192 included.

Moved by decision 0192:
- *Every cell of the probe case within 15 significant digits of Excel* goes to **T-3389 (P1)**. 4 cells are beyond
  15 digits with the right type: fv-nonzero A138, A171, A180 and A185. A112, A153 and A175 are 1 to 2 ulps off,
  which is Phase 1b work (decision 0188 R5).
- *Gate A: 0 points where one side errs and the other is a number* goes to **T-3397 (P0)**. There are 16 cells:
  - 13 the hold lost (sweep A296; fresh A59, A70, A74, A77, A89, A90, A95, A157, A162, A173, A275, A292);
  - 3 neither model gets right (sweep A272; fresh A251, A272).

  RATE's Gate A stays failed until T-3397 settles them; no exception is claimed.
- *Ordinary annuities with fv ≠ 0 across rates 1e-6 to 1 and nper 1 to 1,000 match Excel's result* is met except on
  the cells of the two items above, which those tasks own.

## Probe cells (before the fix)
probes/t3348-rate-fv-nonzero (excel-graph, 2026-10-31), Sheet1!A1:A186, 111 numbers and 75 `#NUM!`:
- A1, A2: the Context example (type 0 and omitted), Excel 0.04310546919282547; the engine gave `#NUM!`.
- A3-A12: the same inputs times 2^-40 ... 2^20 (the secant's trajectory is unchanged, f scales); A13-A23 nper 200 to
  300; A24 type 1; A25-A29 guesses 0, 0.04, 0.05, 0.2, -0.5.
- A30-A186: a seeded sweep of annuities (rate 1E-6 to 1, nper 1 to 1000, both signs of pv and fv, type 0/1, guess
  omitted, 13-digit or cent inputs), stratified by the T-2005 model's outcome (labels in the case's hypotheses): 70
  zero-step stalls, 15 other flat secants, 40 converged, 15 iterates below -1, 12 past the cap, 4 returned at the
  cap, 1 power past the double range.
- Before the fix 60 cells were `#NUM!` in the engine and numbers in Excel (every one a flat secant); the other 126
  matched Excel bit for bit, including the 39 flat secants and the le-1 and cap runs that are `#NUM!` in Excel too.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
Done on the criteria re-scoped by decision 0192 (review round 1). It trades 13 Gate A points for 164 cells of Excel's
type and is reviewed with this task. RATE's Gate A is not met; T-3397 (P0) owns that.

**Cause.** On long annuities with a large balance the residual near the root is rounding noise above 1e-7. The
secant reaches a zero step and then a flat secant (f1 = f0). The T-2005 model stopped there with `#NUM!`, but Excel
returns a value. Every one of the 60 probe misses before the fix was such a flat secant. The sweep showed a second
gap: the model stopped at an iterate below -1, where Excel runs on.

**Fix** (`compute_rate`, financial_util.rs).
- A flat secant holds at r1, the modelled outcome that fits the evidence. The held point returns when it passes the
  convergence test or the cap's test (|f1| < 1e-4, 1 + r1 > 1e-4); otherwise, and on a residual of exactly 0, the
  run is `#NUM!`.
- An iterate at or below -1 runs on, and only the result must be above -1.

`RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)` is 0.04310546919282547 (unit test and
probes/t3348-rate-fv-nonzero A1, A2). The rule and its evidence are in financial.md "RATE holds on a flat secant and
runs on below -1".

**Eval.**
- probes/t3348-rate-fv-nonzero: 126/186 -> 179/186 bit-exact, and 186/186 of Excel's type.
- probes/t3348-rate-below-minus-one: 20/50 -> 50/50.
- The 960-cell fit set stays 960/960.
- `make eval-baseline` with the worktree's verified binary: probes known failures 2260 -> 2267, all 7 in the new
  case, 0 regressions. The baselines only gain lines.

**Reviewer check 7: structural model, kind (c).**

*Hold-out (R4.1).* The split was fixed before any scoring. The cells are the 960 cells of the fit lab's `rate` fit
(RATE_CASES plus rate-step-cap, rate-cap-return-rule and rate-pmt-zero-cap) and the 236 cells of t3348-rate-fv-nonzero
and t3348-rate-below-minus-one, 1196 in all. A cell is hold-out when sha256("T-3348:" + case + "!" + cell)'s first
byte is below 77 (about 30%); that gives 367 hold-out cells and 829 train. The margin is that the new model's
hold-out bit-exact rate is within 5 percentage points of its train rate. Scores are against Excel:

| Candidate | Train bit-exact | Train within 4 ulp | Train within 15 digits | Hold-out bit-exact | Hold-out within 4 ulp | Hold-out within 15 digits |
|---|---|---|---|---|---|---|
| Old model (T-2005) | 765/829 | 765 | 765 | 341/367 | 341 | 341 |
| New model (T-3348) | 822/829 | 825 | 825 | 367/367 | 367 | 367 |
| Correctly rounded (50-digit root) | 75/829 | 150 | 182 | 33/367 | 69 | 81 |

The correctly rounded candidate is generous: where Excel returns a number it takes the real root nearest Excel's
value. Where Excel errs it is `#NUM!` only if no real root above -1 exists. The new model beats it on hold-out by
bit-exact count (367 against 33). Its hold-out rate, 100.0%, is within the margin of its train rate, 99.2%. All 7
misses fall in train (the fv-nonzero cells below).

*Fresh sweep (R4.2) and Gate A envelope (R2).* probes-numeric/t3348-rate-gate-a-fresh was generated after cf5359b54
froze the model, and nothing was tuned on it. It has 300 points, each input log-uniform over the domain (rate 1e-6
to 1, nper 1 to 1000, |pv| and |fv| 1 to 1e12, fv ≠ 0, type 0/1), stratified 6 x 50. The seed is
T-3348/RATE/r4-fresh-after-freeze, the oracle is excel-graph, and the math labels are 50-digit mpmath roots (dps 60
checked at dps 80).

| Stratum | Points | Within 15 digits of Excel | Within 1e-9 of math | Both `#NUM!` | Inside envelope | Outside |
|---|---|---|---|---|---|---|
| rate 1e-6 to 1e-4, balance 1 to 1e6 | 50 | 50 | 7 | 0 | 50 | 0 |
| rate 1e-6 to 1e-4, balance 1e6 to 1e12 | 50 | 43 | 10 | 0 | 43 | 7 |
| rate 1e-4 to 1e-2, balance 1 to 1e6 | 50 | 50 | 44 | 0 | 50 | 0 |
| rate 1e-4 to 1e-2, balance 1e6 to 1e12 | 50 | 46 | 43 | 0 | 47 | 3 |
| rate 1e-2 to 1, balance 1 to 1e6 | 50 | 36 | 35 | 14 | 50 | 0 |
| rate 1e-2 to 1, balance 1e6 to 1e12 | 50 | 30 | 34 | 12 | 46 | 4 |
| **All** | 300 | 255 | 173 | 26 | 286 | 14 |

All 14 outside points are error against number; none is a number outside both bounds:
- A59 `=RATE(2,6278360237.88483,-12528846789.2511,-27774543.9929057,0)`: Excel #NUM!, engine 5.279321035257966e-6, math 0.0000052793235341091736221466966841225504382200871252104 (lost by the hold).
- A70 `=RATE(35,12273893066.0886,-427389409614.586,-2130919589.40718,0)`: Excel #NUM!, engine 8.609994133198454e-6, math 0.0000086099939031526739716825529797870642617165981795346 (lost by the hold).
- A74 `=RATE(321,-2890484105.96678,934408648396.347,-10935378183.8393,1)`: Excel #NUM!, engine 2.8860747367326925e-5, math 0.000028860747367415445305490532355667411365627145473315 (lost by the hold).
- A77 `=RATE(329,-42389848.4869782,13945255025.8794,-23095230.0549541,0)`: Excel #NUM!, engine 1.0450847888000269e-5, math 0.000010450847915839426360174149698560008697101944584973 (lost by the hold).
- A89 `=RATE(14,-240193754.37537,4611780278.38432,-1249534255.10969,0)`: Excel #NUM!, engine 1.0923154117191762e-5, math 0.000010923154164448829573951632792335895891092126276629 (lost by the hold).
- A90 `=RATE(136,1438873775.69802,-196213656238.621,580155950.046744,0)`: Excel #NUM!, engine 3.95617912180478e-6, math 0.0000039561791032256055444758023464441094083470127466901 (lost by the hold).
- A95 `=RATE(2,-2432513870.20006,-3318506.12039095,4868366502.37616,1)`: Excel #NUM!, engine 2.7731842853005824e-6, math 0.0000027731847702484482491057757209744930389246574367448 (lost by the hold).
- A157 `=RATE(1,25550828180.0966,-25201858262.4494,-325879402.767792,0)`: Excel #NUM!, engine 0.0009162227102108508, math 0.00091622271020428869425840433583248302282286727225726 (lost by the hold).
- A162 `=RATE(1,35007935575.2514,-35005912619.7461,-2026687.67999046,1)`: Excel #NUM!, engine 0.0018449121571839915, math 0.0018449119015461378457665992673413857112307145233387 (lost by the hold).
- A173 `=RATE(15,-9926778189.99403,138904429552.499,-209302706.994644,0)`: Excel #NUM!, engine 0.008985792992686202, math 0.0089857929926865893670176712203046477575663825406785 (lost by the hold).
- A251 `=RATE(1,101776720612.892,-676573751810.73,583281411526.675,1)`: Excel 0.014760654402053049, engine #NUM!, math 0.01476065440205248479765663414640382174982854428368 (neither model).
- A272 `=RATE(1,-203158570592.69,3667932.84612167,206903872457.554,1)`: Excel 0.018453750062760996, engine #NUM!, math 0.01845375006276496723821597578547931025033332754635 (neither model).
- A275 `=RATE(2,478550742813.386,-925555108604.956,277863719.42142,1)`: Excel #NUM!, engine 0.07115318239018788, math 0.071153182390187955911759703621808967721482239794923 (lost by the hold).
- A292 `=RATE(6,-101229151545.993,550032163728.64,-13767204.6882258,0)`: Excel #NUM!, engine 0.029098182396110887, math 0.029098182396109697545708752221485019477847367695278 (lost by the hold).

The guard fails. 12 of the points were inside the envelope under the old model, which gave `#NUM!` as Excel does,
and the hold moved them out. Over the whole sweep the old model has 249 inside and the new one 286 (+49, -12).
probes-numeric/t3348-rate-gate-a-sweep, the earlier sweep, is training evidence (A171 and A231 helped build the
rule). There the new model has 298/300 of Excel's type against the old model's 274; it gains 25 and loses 1 (A296),
and 0 points are a number outside both bounds.

**Disposition (decision 0192).**
- The 16 type cells go to T-3397 (P0): the 13 the hold lost (sweep A296 and the 12 fresh points above) and 3 that
  neither model gets right (sweep A272; fresh A251, A272). A296 is `=RATE(351,-1684985188.67608,607019725001.509,
  -15796156082.0366,0)`: Excel `#NUM!`, engine 1.8814890660971136e-6, math
  0.0000018814890215042125897557712909148668011313466560856. Sweep A272 is `=RATE(4,-70299017520.6175,
  254784242189.789,-1503672.27062336,0)`: Excel 0.040657819686895275, engine `#NUM!`, math
  0.040657819686896052522303945460859198781648510530346.
- No rule on the hold gives Excel's type on all the holds: an absolute bound, a bound relative to the residual's
  terms, zero-step against distinct iterates, and a step cut all fail (financial.md). The 16 are the noise floor's
  path.
- The 7 fv-nonzero cells of the right type that are not Excel's bits are known failures. T-3389, raised to P1, owns
  the 4 beyond 15 digits (A138, A171, A180, A185); A112, A153 and A175 are 1 to 2 ulps off, which is Phase 1b work
  (decision 0188 R5).

**Build note.** The shared CARGO_TARGET_DIR can link another worktree's engine, and the Makefile's
`target/release/rustsheet-eval` was a stale hard link. Every eval number above comes from a `-v` rebuild that
compiled this worktree's engine, copied with `cp --remove-destination` and checked with `show`. The old model,
the hold-out and the fresh-sweep comparison ran in temporary unit tests (removed) that call `compute_rate` and
main's copy of it.
