---
id: T-3770
title: "bup's prefactor: score one register rounding against two stored ops on bup's own callers (L140 bup+bpser, n > 1)"
milestone: M9
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
One evidenced rounding model for TOMS bup's prefactor P·x·y/a in Excel: either `bup`'s two stored roundings
(fl(fl(P·fl(x·y))/a)) or T-3382's one register rounding (fl(x87(P·fl(x·y)/a))) on every caller, or a recorded reason
why Excel's one bup routine rounds differently by caller.

## Context
T-3382 (PR #2776) rounds W = bup(b0', a0, y0, x0, 1) once in a register on L140 bup+bgrat, where that form wins T-2512's
hold-out (1,306 of 1,613 bit-exact against 1,046 stored twice; docs/research/t2464/r3-bgrat/bupdens.out). `bratio/bup.rs`
`bup` (L140 bup+bpser and the n > 1 paths) stores twice: `bratio_excel.py`'s `bup_arith = "w4"` covers only the W loop, so
the prefactor's rounding was never scored on those callers (Excel-fidelity review of #2776). Score both forms on the
existing probes-numeric cells of those branches (bratio's `Branch::L140BupBpser`, `L140BupBupBgrat`), with a hold-out split
fixed before scoring.

## Acceptance criteria
- [ ] Both prefactor forms scored (exact / within 4 ulp / within 15 digits, tune and hold-out) on `bup`'s callers; the
      result recorded in statistics.md "Incomplete beta: the integrated model".
- [ ] If the register form wins, `bup` and `l140_bup_bgrat` share one prefactor helper; no gated or probes-numeric cell
      goes from within 15 digits to visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
