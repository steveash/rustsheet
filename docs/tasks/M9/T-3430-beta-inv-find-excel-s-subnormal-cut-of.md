---
id: T-3430
title: "BETA.INV: model where Excel's AS 109 search fails at tiny shapes (1 or 0 versus #NUM!)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BETA.INV and BETAINV give Excel's 1 or `#NUM!` (and the lower tail's 0 or `#NUM!`) where the flipped quantile y is a
subnormal double, instead of T-3351's rule that a subnormal y is `#NUM!`.

## Context
- T-3351 (probes/t3351-beta-inv-p-near-1, excel-graph) found that BETA.INV(p, alpha, beta) for p above 1/2 is 1 - y,
  with y the lower quantile of Beta(beta, alpha) at 1 - p, and `#NUM!` once Excel's search for y fails. The failure
  is inside the subnormals and depends on alpha: with beta = 0.03, Excel gives 1 at y = 2.4E-309 (alpha 10, C199),
  1.5E-308 (alpha 2, A199) and 1.4E-317 (alpha 0.5, A211), and `#NUM!` at y = 5.6E-312 (C200), 3.4E-311 (A200) and
  3.3E-320 (A212) (y from 50-digit mpmath). The lower tail behaves the same way: `BETA.INV(6E-10,0.03,2)` is 0 (a
  subnormal shown as 0) and `BETA.INV(5E-10,0.03,2)` is `#NUM!` (B199, B200).
- For alpha 0.5 the cut sits where y^(beta-1) overflows (about 1.3E-318 at beta 0.03), which suggests Excel's
  Newton step goes non-finite there. For alpha >= 1 the cut is near 1E-310, which that does not explain.
- T-3351 treats any subnormal y as `#NUM!`. That mismatches A199, A209, A210, A211 and C199, and the lower tail's
  B199, B209, B210 and B211 (if those are not already right).
- Same probe, lower tail at a normal y: B197, B198, B206, B207, B208 (`BETA.INV(8E-10,0.03,2)` and its
  neighbours, y 1E-303 to 1E-307) differ from Excel at about 1E-13 relative. Both sides are close to 50-digit mpmath:
  Excel is 1.5E-14 to 7.2E-14 off it, ours about 9E-14 (B197). Take these too if the cut's mechanism explains them.
  Otherwise move them to a Phase 1b follow-up.
- The cut is not just "y underflows". T-3351's fresh sweep (probes-numeric/t3351-beta-inv-sweep, excel-graph) has
  Excel at 1 where the true y underflows far below the subnormals, and ours at `#NUM!`:
  A88 `BETA.INV(1-1.5527583056201E-13,0.208134958526082,0.0396646084111499)`,
  A89 `BETA.INV(1-4.25367275749165E-09,0.147200045247935,0.013385366393858)` (y about 1E-620) and
  A122 `BETA.INV(1-4.7838841265984E-10,2.52340266710761,0.0133219075721009)` (y about 1E-700). The probe grid has the
  opposite at beta 0.01 (`BETA.INV(1-1E-8,2,0.01)` is `#NUM!`). So Excel's AS 109 Newton search (its acu, step
  halving and start clamp) decides where it fails, not the size of y. The lower tail has the same family:
  A175 `BETA.INV(1.1340254693883E-09,0.0675423557406452,0.00285138595850823)` is `#NUM!` in Excel and ours is the true
  1.49E-112, and A188 `BETA.INV(2.29593712441749E-10,0.00141043194789831,3523.31248983393)` is 1.366822740515963E-153 in
  Excel (true y 1.3E-6838) where ours is `#NUM!`.
- Code: `upper_quantile_near_one` in crates/rustsheet-engine/src/functions/statistical/beta.rs.

## Acceptance criteria
- [ ] A probe that brackets the cut on a grid of alpha and beta (excel-graph), with its mechanism in
      docs/excel-semantics/statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)".
- [ ] The cells above match (or the B-column normal-y cells move to a follow-up). Any threshold chosen by scoring follows decision 0188 R4 (hold-out, fresh sweep,
      `FITTED`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Normal-range values (T-3351).
