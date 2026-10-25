---
id: T-2505
title: "Incomplete beta: bratio dispatch and Excel's bpser branches for every caller (T-2464 E3)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2633, T-2502, T-2504]
---

## Goal
The engine's incomplete beta uses TOMS 708 bratio's dispatch with x0 >= 0.3, and computes every bpser branch with
Excel's series and prefactor. The callers are BETA.DIST (cdf), F.DIST, F.DIST.RT, FDIST, BINOM.DIST and NEGBINOM.DIST,
and the branches are w_bpser, w1_bpser and bpser(a,b>1). Other branches keep today's kernel until their tasks land.

## Context
- T-2464 research, step E3 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rules are in docs/excel-semantics/statistics.md: "Incomplete beta is TOMS 708 `bratio`" (dispatch, 0.3, complement
  (0.5 - w) + 0.5, caller arguments) and "Incomplete beta bpser".
- The bpser itself, with x87 stored ops:
  - s = fl(1/a), with forward terms c·((0.5 - b/n) + 0.5)·x and s += c/(a+n), until s stops changing;
  - I = fl(EXP(z)·s), with z = fl64(fl(a·LN x) - L) and EXP of the extended argument (T-2633);
  - L is r2-bpser-small's GL53 form, fl(GL(a0) + fl(GL(b0) - GL(fl(a0+b0)))), for b0 < 8;
  - L = fl(GL(a0) + algdiv(a0, b0)) for b0 >= 8;
  - no fpser; apser falls through the dispatch.
- r2-bpser-small's GL53 beats r2-bratio's BEST on its own 2,000-shape probe (t2464-r2-bpser-small-shapes). It loses on
  the old 1 < b0 < 8 cells: 3,365 against 4,038 of 7,455, mostly w2f (0.5, 6.25) and w4p1 (2.5, 0.5)
  (verify-r2-bpser-small/old_sets.out).
- So: implement GL53. Score both L forms (GL53 and r2-bratio's w3_lg = toms, fl(gamln1(v) - LN v) for v < 1) on every
  set, and record both in the Outcome.
- The engine's GAMMALN model is exact on only 274 of 771 shape arguments in [1, 2), and on 1,738 of 2,585 below 1
  (t2464-r2-bpser-small-gl). Bit-exact counts depend on it (T-2122). r2-bratio's ablation says the cdf moves by only
  -4 to +12 cells.
- Models: `r2-bratio/bratio_excel.py` (`bratio`, `bpser_w3`), `r2-bpser-small/lib.py` (`V(nofpser=True,
  noapser=True, lg="GL53/glalgdiv")`), `w3-series/model.py`.
- r2-impact's expected change (ladder step S3):
  - -632 visibly-off pre-T2464 cells (-303 hold-out; -354 outside BETA.DIST) and -5,281 in the w2f probes;
  - 22 broke;
  - 2 gated bit-exact cells lost: gen-m9 F.DIST.RT 0a614cfe A19 and FDIST 05d44f6d A175.
- Subsumes T-2119's main change: BETA.DIST's cdf leaves statrs for the engine kernel, now the bratio port.
  - T-2119's nine chained BETA.INV cells (BETADIST_BETAINV.xlsx) move here. Settle them under decision 0083's
    allowance, as T-2119 says.
  - When this task lands, mark T-2119 dropped (superseded), or narrow it to whatever is left.
  - F.INV and F.INV.RT inverting the same cdf is T-2511.
- Related: T-2146 (BINOM/NEGBINOM last digits) and T-2219 (F to 15 digits) own cells that this task moves. Re-triage
  them when it lands.

## Acceptance criteria
- [ ] Dispatch: on the 434 differing cells of t2464-r2-bratio-x29 in [0.29, 0.3), the engine takes the 0.3 branch, and
      at least 284 are bit-exact.
- [ ] bpser on t2464-r2-bpser-small-shapes is within 15 digits on 3,083 of 3,083 (b0 <= 1), 4,602 of 4,602
      (1 < b0 < 8) and at least 2,861 of 2,868 (b0 >= 8).
- [ ] r2-bratio's per-branch bit-exact counts are met or beaten: w_bpser 6,767 of 11,845, w1_bpser 1,666 of 1,845,
      bpser(a,b>1) 978 of 1,523. Both L forms are reported.
- [ ] BETA.DIST's cdf no longer calls statrs.
- [ ] The two gated lost cells and T-2119's chained BETA.INV cells are recorded under decision 0083 (or a decision
      record). No other baseline line is added.
- [ ] statistics.md's bpser and bratio entries gain "Implemented in" lines.
- [ ] The T-2464 research harness is deleted: `functions/statistical/t2464_harness.rs` and its `mod t2464_harness;`
      line in `functions/statistical/mod.rs` (docs/research/t2464/README.md "Rebuilding the harness" then points at the
      engine's own kernel, or the research tools keep a copy outside the crate).
- [ ] `make check` green; reviewers approve.

## Out of scope
- L140 (T-2506), bfrac/basym (T-2508), the bgrat family (T-2509, T-2510) and the inverses (T-2511).
- The T tails stay on `student_t.rs` until T-2509.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns, a mapped cell in a bpser branch, is within half a unit of the
15th significant digit of Excel's value. r2-impact counts 746 such cells visibly off pre-T2464 (bpser(a,b>1) 369,
w_bpser 341, w1_bpser 36); the model leaves 93, 48 of them BINOM in bpser(a,b>1). Cells still off then are named in
the Outcome and handed to T-2512. Bit-level criteria still unmet move to a follow-up task without the campaign key.
