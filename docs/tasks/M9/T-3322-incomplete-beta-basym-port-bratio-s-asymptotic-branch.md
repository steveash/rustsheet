---
id: T-3322
title: "Incomplete beta basym: port bratio's asymptotic branch for a0, b0 > 100 near the mean (T-2508 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2508]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
bratio's basym branch (a0, b0 > 100 and λ <= 0.03·min(a0, b0)) is ported in `bratio/`, so BETA.DIST, F.DIST,
T.DIST and BINOM.DIST points near the mean of two large shapes leave today's Cephes kernel.

## Context
- T-2508 ported bfrac (b0 >= 40) and left basym on today's kernel (`Branch::Basym` falls through `ported_tails`;
  `bratio/tests.rs` `unported_points_keep_todays_kernel` pins (0.5, 500, 600)).
- Model: docs/research/t2464/r2-bratio/bratio_excel.py `basym` (it needs `rlog1`, `erfc1(1, ·)` and `bcorr`; the
  engine has `incomplete_beta::rlog1` and `bratio/ln_beta.rs` `bcorr`; search before writing any helper).
- No eval case is known to route to basym yet: find them first (e.g. a probes-numeric sweep at shapes above 100 near
  x = a/(a+b), or a BETA.INV round trip there); if none exists, add a probe case with the `excel-probe` skill.
- statistics.md "Incomplete beta L140 and bfrac" holds the prefactor evidence (brcomp = Excel's density·x·y).

## Acceptance criteria
- [ ] `Branch::Basym` points are computed by the port; a unit test pins Excel's oracle values for at least three
      basym cells (bit-exact or the ulp distance stated), and the unported-kernel test no longer lists basym.
- [ ] No gated known-failure line is added; the visibly-off count of the basym cells does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac's bit-level scores (T-3323).
- The fraction's own arithmetic (T-2512).
