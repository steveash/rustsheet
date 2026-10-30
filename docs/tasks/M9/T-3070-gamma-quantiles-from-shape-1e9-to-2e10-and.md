---
id: T-3070
title: "Gamma quantiles from shape 1E9 to 2E10 and above the chi-square df cap (T-2982 rest: E15, A2-A4, the #NUM! band, CHIINV at 2E13, the 1E11 z^2 term)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2982]
campaign: visible
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

## Stage (decision 0114)
`campaign: visible`, not `accuracy` (re-staged 2026-10-30 by this task). The accuracy stage needs Excel close to
correctly rounded, and from a shape of 1E9 it is not: its quantile is the true one only inside a band, `#NUM!` outside
it, and a Wilson-Hilferty closed form about 1.000266 times the true quantile (2.66E-4 relative, 10^11 ulps at 1E13)
where the start's density underflows (probes/t3070-gamma-inv-band, probes/t2982-gamma-inv-huge-shape;
docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"). Matching it means reproducing
Excel's algorithm, so the target is visible parity and the closed form's digits are a fit.

## Acceptance criteria
- [x] The cells above pass or are named with their ulp in the Outcome; no new known-failure lines (the new probe's
      139 failing lines are named and owned by T-3272).
- [x] `make check` green; reviewers approve (review not run here: the caller runs it).

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- gen-m9/8f2ac25945af87da Calc!A240 `CHISQ.INV(1E-16,10000000000.5)`: Excel 9998837266.383266, ours `#NUM!` (the lower-tail twin of A89 and A90's CHISQ.INV.RT at 10000000000.5 df).

## Outcome
- Probe probes/t3070-gamma-inv-band (381 cells, excel-graph, 2026-10-30): p sweeps at shapes 1E9 to 1E12, the median
  at 17 shapes of the band, the chi-square functions from 1.5E10 to 1E15 degrees of freedom, CHISQ.INV and
  CHISQ.INV.RT at 2E9 to 1E10. It shows three regimes (docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at
  a huge shape", rewritten): Excel's Wilson-Hilferty start itself where the density there underflows; the true
  quantile inside a band of the start's offset and z; `#NUM!` elsewhere. CHISQ.INV.RT and CHIINV take any df.
- New `incomplete_gamma/huge_shape.rs` `huge_shape_quantile` (GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT, CHIINV from
  shape 1E9; the band edges are fits inside their brackets), the true quantile by halving the whole range (statrs'
  search is NaN or far off there; `REFINE_SHAPE_MAX` and `WILSON_HILFERTY_SHAPE_MIN` are gone); the closed form refitted
  with s and terms in 1/a, z^2/a and z^3/a^(3/2) (within 4 ulps from 1.5E10 to 1E300; the 1E11 z^2 term was 100 to
  12,459 ulps); `chisq.rs` lets the right tail past `MAX_DEGREES_OF_FREEDOM`.
- Stage: retagged `campaign: visible` (Stage section): Excel is the inaccurate side.
- Main-vs-HEAD per-cell comparison over every suite (main b666cbc85 binary against this branch, both with the new probe):
  fixed (now bit-exact or within the harness's pass) 105 (102 probes, 3 gen-m9); lost bit-exact 0; newly failing 0;
  newly visibly off 0; type or error changes 0; visibly fixed 246 (gen-m9 4e55500bb8ad3cd8 A341, 7bd219ec0c0c5e22
  A239, 8f2ac25945af87da A240, a013a45a8fbcab4f A338; probes-numeric m9-legacy-chiinv A85, A86, t1049-chisqinv A27,
  t1049-chisqinvrt A27; incomplete-gamma-cap E15; t2982 A2-A4, A7-A10, A24, A25, A44, A60-A77, A89, A90, A92, A93; the
  rest the new probe's). Still failing and moved: 60 closer, 2 farther (t3070 A179 1 -> 5 ulps, t2982 A29 1 -> 4).
  Baseline: gen-m9 3151 -> 3148, probes 2316 -> 2443 (12 fixed, 139 new-coverage lines of the new probe), no
  regressions; visible parity 2542 in-scope visibly off (1 in the new case).
- Left (T-3272): t3070 A371 `CHISQ.INV.RT(1E-100,1E10)` 10005667524.01516 against 10005667524.015223 (33 ulps, the one
  visibly-off cell), A214 21 ulps, A229 11, A110 9, the rest 1 to 6 ulps (median 1); t2982 inversions 1 to 5 ulps;
  incomplete-gamma-cap E5, E14, E19 1 ulp each. Not inversions, so not done here: t2982 A144-A146 (Temme's tails, 9, 19
  and 67 ulps) and E13 (2 ulps), T-0971's; the GAMMA.DIST `#NUM!` rules (t2435 A25, A26, A101, A102, gen-m9
  2801c1c11c21e134 A241, 40425341c5034eed A265) moved to T-3271.
- T-1891's 25 gen-m9 lines: the huge-shape ones pass (4e55500bb8ad3cd8 A341, 7bd219ec0c0c5e22 A239, 8f2ac25945af87da
  A240) or are within 15 digits (a013a45a8fbcab4f A338, 1 ulp class; 0a614cfe974b59c3 A327, 165db58d1540f014 A209,
  85f48d5cdfb24555 A382, a7bf0d8d81d33ba6 A129, bc40d37cb4d8301d A371, c18dbf21f11093f3 A119, ecc87f80077959e0 A99 1 to
  2 ulps, unchanged); the small-shape inversions 146b46e74f2dac3b A152, 60b0b6e69ad35a45 A210, 984cba67bbe6972b A43,
  c2974c061384fdd9 A179 and caa1eed48b1883fe A282 stay T-0972's and T-3013's (unchanged from main).
- Attempts (decision 0056): 1, the classifier on the probe grid (CF by density underflow, band by z + k o and o - z;
  k from 1.30 to 1.41 separate every cell); 2, the closed form's terms (polynomial in z/sqrt(a), least squares on 167
  cells). Excel's search itself is still unknown.
