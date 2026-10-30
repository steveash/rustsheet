---
id: T-3234
title: "GAMMA.DIST, GAMMADIST and CHISQ.DIST cdf and density last digits on the m9, t1048/t1049 and t2753 probes (199 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2638, T-2852, T-3060, T-3147]
defer: phase-1b
---

## Goal
GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHIDIST and CHISQ.DIST.RT match Excel to the bit on the 199 incomplete-gamma cdf
and gamma density cells listed below. Each cell is already within half a unit of Excel's 15th significant digit, and
no open task lists any of them.

## Context
- Gap triage 2026-10-30 (`make eval` at e1255db9, excel-graph evidence). Each cell has one of these functions as its
  only non-glue call. By function: GAMMA.DIST 119, CHISQ.DIST 59, GAMMADIST 18, CHIDIST 2, CHISQ.DIST.RT 1. By suite:
  probes-numeric 142, probes 36, gen-m9 11, ironcalc-calc 10.
- Ulp distance, ours against Excel's: 1 ulp 139, 2 ulp 28, 3 ulp 16, 4 to 7 ulp 11, 12 to 19 ulp 5. The verdicts are
  187 `NumericUlp` and 12 `NumericClose`, and none is visibly off.
- Neighbouring tasks, none of which lists these cells:
  - T-2638 (accuracy) owns the visibly-off tails and densities of the gen-m9 sweep, plus nine named
    cells of the T-2753 density-ratio sweep.
  - T-2852 owns the density between x/beta 1E-15 and 1E-10, and at shape 50.
  - T-3060 owns the cdf at a shape below 1 and a tiny x.
  - T-3147 owns CHISQ.DIST's large-shape prefactor at 50 and 100 df.
  - T-2695 (deferred) owns a separate set of gen-m9 CHIDIST and CHISQ.DIST cells.
  - T-2367 (deferred) owns POISSON.DIST D16 and GAMMA.INV F38.
  The first four change the kernel these cells use, so re-run this list after them before any fit.
- Where Excel is the inexact one. Exact values come from 50-digit decimal arithmetic on integer-shape closed forms,
  and Excel is always right, so these are its formulation to reproduce:
  - probes-numeric/t1049-chisqdist Calc!A24 `=CHISQ.DIST(1,20,FALSE)` = e^-0.5/(2^10·9!): Excel
    1.6322616219566172e-09, ours 1.6322616219566208e-09. Ours is the exact value; Excel is 17 ulp off it.
  - probes-numeric/t1049-chisqdist Calc!A15 `=CHISQ.DIST(0.5,20,TRUE)` = P(10, 0.25): Excel 2.0942485399973563e-13,
    ours 2.094248539997361e-13. Ours is exact again; Excel is 19 ulp off.
  - probes-numeric/t1048-gammadist Calc!A5 `=GAMMADIST(3,2,1.5,TRUE)` = 1 - 3e^-2: Excel 0.5939941502901619, ours
    0.5939941502901618. Here Excel is the exact one and ours is 1 ulp off.
  So the target mixes Excel's own density and prefactor formulation (statistics.md "One incomplete-gamma kernel under
  ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)", "Incomplete gamma: accurate kernel (T-0971)", "GAMMA.DIST's
  density at a tiny x / beta is a log-space form (T-2753)") with the engine's own last-ulp error. This is numeric
  fitting, so opus. Code: `gamma_prefactor`, `gamma_density` and `incomplete_gamma_pair` in
  crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs, called from gamma.rs and chisq.rs.
- Evidence: the cached cases in the list below give 199 failing points and several hundred passing ones, at shapes
  0.5, 1, 2, 9, 50 and 500. If they still cannot separate the candidate formulations, do not fit blind. First file a
  research-lane oracle sweep of GAMMA.DIST(x,a,1,FALSE) and (..,TRUE) over integer and half-integer shapes 1 to 20,
  and depend on it.

### Cells owned
Each cell below has GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHIDIST or CHISQ.DIST.RT as its only non-glue call:
- probes/t2753-gamma-density-ratio-sweep.case.json (34): S!A1, A6, A8, A9, A10, A11, A16, A17, A19, A21, A22, A23, A25, A27, A30, A31, A32, A33, A35, A36, B8, B9, B10, B11, B16, B17, B18, B19, B22, B23, B30, B32, B33, B36
- probes-numeric/m9-chisq-dist.case.json (30): Calc!A4, A6, A8, A10, A12, A18, A22, A23, A24, A25, A26, A27, A28, A30, A32, A33, A34, A37, A38, A40, A41, A42, A48, A49, A53, A56, A65, A66, A68, A70
- probes-numeric/t1049-chisqdist.case.json (28): Calc!A5, A6, A7, A8, A10, A14, A15, A18, A21, A22, A23, A24, A26, A29, A30, A31, A34, A36, A37, A38, A42, A44, A45, A55, A58, A59, A62, A63
- probes-numeric/t1048-gammadist.case.json (24): Calc!A5, A6, A11, A12, A13, A14, A21, A22, A24, A25, A26, A27, B5, B6, B11, B12, B13, B14, B21, B22, B24, B25, B26, B27
- probes-numeric/m9-gamma-dist-a9.case.json (19): Calc!A3, A4, A5, A6, A7, A8, A10, A12, A13, A14, A15, A16, A17, A18, A20, A23, A24, A25, A27
- probes-numeric/m9-gamma-dist-a2.case.json (14): Calc!A3, A4, A5, A6, A10, A13, A14, A15, A16, A20, A23, A25, A28, A30
- probes-numeric/m9-gamma-dist-a50.case.json (10): Calc!A5, A6, A8, A10, A15, A16, A18, A20, A25, A26
- probes-numeric/m9-gamma-dist-a0p5.case.json (8): Calc!A4, A6, A10, A14, A16, A20, A28, A30
- ironcalc-calc/STATISTICAL/GAMMA.xlsx (7): GAMMA.DIST GAMMA.INV!E7, E8, E11, E21, E23, E38, E40
- probes-numeric/m9-gamma-dist-a500.case.json (6): Calc!A5, A8, A15, A18, A25, A28
- probes-numeric/m9-gamma-dist-a1.case.json (3): Calc!A10, A20, A30
- ironcalc-calc/STATISTICAL/CHISQ.xlsx (2): Sheet1!E4, G2
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A60
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A295
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A308
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A220
- gen-m9/530c636749a85209.case.json (1): Calc!A242
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A257
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A30
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A171
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A2
- gen-m9/cde551ea787f713a.case.json (1): Calc!A390
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A174
- ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx (1): BETA_DIST!L4
- probes/incomplete-gamma-cap.case.json (1): Sheet1!E13
- probes/t2738-gamma-dist-tiny-ratio.case.json (1): S!A4

## Acceptance criteria
- [ ] These known-failure lines disappear from the baselines, with no new ones in any baseline (`make eval-baseline`,
      `make eval-check`: 0 regressions):
      - the 11 gen-m9 lines from eval/baseline/gen-m9.txt;
      - the 10 ironcalc-calc lines from eval/baseline/ironcalc-calc.txt;
      - the 36 probes lines from eval/baseline/probes.txt.
- [ ] probes-numeric is ungated, so its 142 owned cells are scored from `make eval`'s report.json: they pass
      bit-exact. Or, under the decision 0056 time box, the Outcome records the formulations tried with their bit-exact
      counts, and the open cells move to a follow-up.
- [ ] docs/excel-semantics/statistics.md records the density and prefactor formulation found, citing the three
      "Excel is the inexact one" cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off gamma cdf and density cells (T-2638, T-2852, T-3060, T-3147).
- The gamma quantiles (T-3232) and POISSON.DIST (T-2055).
