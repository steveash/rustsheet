---
id: T-3070
title: "Gamma quantiles from shape 1E9 to 2E10 and above the chi-square df cap (T-2982 rest: E15, A2-A4, the #NUM! band, CHIINV at 2E13, the 1E11 z^2 term)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2982]
campaign: accuracy
---

## Goal
GAMMA.INV, CHISQ.INV, CHISQ.INV.RT and CHIINV give Excel's value (or its `#NUM!`) at the shapes T-2982 left on
statrs' search, and the closed form's last digits at 1E11 are Excel's.

## Context
- T-2982 found Excel's quantile from shape 2E10 is a Wilson-Hilferty closed form with Excel's constants
  (`incomplete_gamma` `wilson_hilferty_quantile`; docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at a
  huge shape") and moved GAMMA.INV/GAMMAINV onto it. Left open:
  - shapes 1E9 to 5E9, where Excel is the true quantile and ours `#NUM!` (statrs' search above `REFINE_SHAPE_MAX`):
    probes/t2982-gamma-inv-huge-shape A2-A4, probes/incomplete-gamma-cap E15 (`CHISQ.INV(5.6221E-26,4715100000)`);
    re-check the gen-m9 cells that pin `#NUM!` above 1E9 before moving the cut;
  - the band 7E9 to 1E10 where Excel is mostly `#NUM!` but A44 `GAMMA.INV(1E-30,1E10,1)` = 9998853641.00538, and
    where between 1E10 and 2E10 the closed form starts (probe it);
  - CHISQ.INV.RT and CHIINV above `chisq.rs` `MAX_DEGREES_OF_FREEDOM` (1E10): Excel's `CHIINV(0.5,2E13)` =
    `CHISQ.INV.RT(0.5,2E13)` = 20005316471010.844 (A92, A93), the closed form at scale 2; `CHISQ.INV(0.5,2E13)` is
    `#NUM!` (A91). Probe other p and which other chi-square functions take df above 1E10;
  - at shape 1E11 the fit is 100 to 1,500 ulps off (A60-A77, a z^2 term of about 2E-4 z^2 absolute and a constant of
    about -2.3E-3 at z = 0 for shapes 2E10 to 1E11, A7-A9): find the exact form; A24 and A60 (p = 1E-300) are about
    250 ulps off at 1E13 and 1E11;
  - Temme's tails at the 1E8 cut, A144-A146 (about 1E-15 relative), and E13, E14, E19 (1 ulp each).
  - Every known-failure line of probes/t2982-gamma-inv-huge-shape is this task's (colon spans, decision 0136):
    - probes/t2982-gamma-inv-huge-shape A1:A4, A7:A13, A16, A19, A22:A30, A34, A36:A41, A44;
    - probes/t2982-gamma-inv-huge-shape A60:A85 (the closed form's last digits, 1 to 1,546 ulps);
    - probes/t2982-gamma-inv-huge-shape A89, A90 (CHISQ.INV.RT at 10000000000.5 df, ours `#NUM!`), A92, A93, A144:A146;
    and T-2982's owned gen-m9 cells (the 25 lines listed in
    docs/tasks/M9/T-1891-gamma-dist-poisson-and-the-gamma-inversions-give.md) and t2435 cells (gen-m9 2801c1c11c21e134
    A241, 40425341c5034eed A265; probes/t2435-dist-extremes A25, A26, A101, A102), leaving any small-shape inversion's
    last digits to T-0972.

## Acceptance criteria
- [ ] The cells above pass or are named with their ulp in the Outcome; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- gen-m9/8f2ac25945af87da Calc!A240 `CHISQ.INV(1E-16,10000000000.5)`: Excel 9998837266.383266, ours `#NUM!` (the lower-tail twin of A89 and A90's CHISQ.INV.RT at 10000000000.5 df).
