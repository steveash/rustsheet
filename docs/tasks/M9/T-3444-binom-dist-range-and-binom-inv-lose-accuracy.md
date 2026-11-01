---
id: T-3444
title: "BINOM.DIST.RANGE and BINOM.INV lose accuracy at a large n (statrs past the walk budget)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BINOM.DIST.RANGE and BINOM.INV (and CRITBINOM) at n from 6E5 to 2^31 - 2 are within 15 digits of Excel, as BINOM.DIST
is since T-3352, instead of grossly wrong.

## Context
- Found by T-3352: past the double-double walk's 2^10 terms (`discrete.rs` `binomial_range`) BINOM.DIST.RANGE falls back
  to statrs' cdf difference, and BINOM.INV's bisection crosses statrs' cdf throughout. Near the mean at a large n statrs'
  cdf is grossly wrong: `BINOM.DIST.RANGE(1000000000,0.3,300000000,300000100)` is 1.47 (a probability above 1) and
  `BINOM.INV(1000000000,0.3,0.5)` 299998246 (the median is 300000000). statistics.md "BINOM.DIST and NEGBINOM.DIST at a
  large n: basym and Loader, within 1E-11 of the math (T-3352)".
- BINOM.DIST's cdf now takes the bratio port with basym (`binom.rs` `binom_dist_of`, `bratio::basym_tails`), and its pmf
  past the walk Loader's kernel (`binom.rs` `loader_pmf`). Reuse them; Excel's form for BINOM.DIST.RANGE (a cdf
  difference or a sum of pmfs) and BINOM.INV's search are not known: probe them first (excel-probe).
- `binom_inv_of`'s 31-step bisection and its lifting weight must stay bounded (directive 5).

## Acceptance criteria
- [x] A probe of BINOM.DIST.RANGE and BINOM.INV at n 1E4 to 2E9 (oracle-filled) is within 15 digits of Excel, with the
      right type and error. BINOM.INV and CRITBINOM: 252 of 252 cells. BINOM.DIST.RANGE: every row now takes Excel's
      form (one point the pmf, from 0 the cdf, else the cdf difference, past the mean the upper-tail difference); the
      cells still off miss by BINOM.DIST's own last digits (49, now T-3447's) or the upper-tail form's (21, T-3740).
- [x] Gate A floor on a fresh sweep (decision 0188 R2): split out to T-3741 (Definitions for BINOM.DIST.RANGE and
      BINOM.INV, as T-3553 was split out of T-3358).
- [x] No gated cell regresses; the rule is in docs/excel-semantics/statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST and NEGBINOM.DIST (T-3352).

## Handoff (resumed)
Superseded by the Outcome below: every "Not done" item was finished or split out (T-3740, T-3741).

Opus session ran out of its time budget mid-task (2026-11-01). State of `agent/functions-stat/T-3444-binom-large-n`:
- Probe probes/t3444-binom-range-inv-large-n (756 cells, excel-graph): Excel's BINOM.INV/CRITBINOM is the smallest k
  whose own BINOM.DIST cdf reaches alpha (126/126 rows); BINOM.DIST.RANGE is its own cdf difference on 105/126 rows,
  a one-point range is the pmf, and the 21 rows from mean + 3 sd to mean + 5 sd use an unidentified form (2E-16 to
  6E-14 off the difference). Rule recorded in statistics.md "BINOM.DIST.RANGE and BINOM.INV at a large n sit on
  BINOM.DIST's own cdf (T-3444)".
- Code: `binom.rs` `excel_binom_cdf` and `excel_binom_pmf` (BINOM.DIST's own paths, extracted); BINOM.INV bisects on
  the cdf; RANGE takes pmf / cdf-from-0 / walk / cdf difference. Probe 490/756 pass (all 252 INV/CRITBINOM cells);
  `make eval-baseline`: gen-m9 -3, ironcalc-calc -6, probes -3 known failures, 0 regressions, +266 new-coverage lines
  (the new case: BINOM.DIST's own last digits, T-3447, and the upper-tail RANGE rows).
- Not done: (1) `make check` result (started, see the next commit or re-run); (2) BINOM.INV's per-call cost: each of
  its 31 steps may now walk 2^10 terms or run bratio, so re-measure and re-size the `Weighted(4096)` lift charge in
  lift/kinds/statistical.rs `BINOM_INV` and the doc comment on `fn_binom_inv` (directive 5); (3) the upper-tail RANGE
  form (try Q(s1 - 1) - Q(s2) from bratio's other tail) or file it as a follow-up; (4) the Gate A fresh-sweep criterion;
  (5) the review panel (security, architecture, quality, excel-fidelity), then PR and mark-ready.

## Outcome
- Excel's BINOM.INV/CRITBINOM is the smallest k whose own BINOM.DIST cdf reaches alpha, and BINOM.DIST.RANGE is built
  on BINOM.DIST's own pmf and cdf (statistics.md "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own
  cdf (T-3444)"). `binom.rs` `excel_binom_pmf`, `excel_binom_cdf` and `excel_binom_sf` are BINOM.DIST's paths, shared;
  BINOM.INV bisects on the cdf; RANGE takes the pmf for one point, the cdf from 0, the walk, then past the mean the
  difference of upper tails (18 of the 21 rows where Excel is not its own cdf difference go from up to 6.5E-14 to
  within 1E-15), else the cdf difference. `BINOM.INV(1000000000,0.3,0.5)` is 300000000 (statrs' 299998246 before).
- New probe probes/t3444-binom-range-inv-large-n (756 cells, excel-graph): 490 pass, all 252 BINOM.INV/CRITBINOM.
  Eval delta (`make eval-baseline`): gen-m9 -3, ironcalc-calc -6, probes -3 known failures, 0 regressions; the new
  case's 266 failing cells are owned by T-3447 (BINOM.DIST's last digits) and T-3740 (the upper-tail form).
- Directive 5: BINOM.INV's search now runs BINOM.DIST's cdf (bratio, else the 2^10-term walk) per step, measured in
  release at up to about 200 us an element (was 65 us; grid n 1E3 to 2^31 - 2, p 1E-9 to 1 - 1E-6, alpha 1E-300 to
  1 - 1E-12), so its lift weight goes from 4096 to 16384: a lifted call is capped at 8,192 elements (~1.6 s), the
  DEVIATION in statistics.md updated. BINOM.DIST.RANGE keeps 4096: past the walk it adds at most two bratio tails
  (a few us each) to the walk's bound.
- Follow-ups: T-3740 (the exact upper-tail form), T-3741 (Gate A tables, the fresh-sweep criterion).
