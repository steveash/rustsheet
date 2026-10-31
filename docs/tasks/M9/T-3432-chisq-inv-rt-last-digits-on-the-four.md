---
id: T-3432
title: "CHISQ.INV.RT last digits on the four t2561 #NUM!-boundary probes (419 cells, 1 to 4 ulp, within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
CHISQ.INV.RT is bit-exact with Excel on the right-tail quantile cells of T-2561's four #NUM!-boundary probes
(p from 1E-300 to 1E-20, df 4 to 50). The engine is 1 to 4 ulp off there. Excel is the correctly rounded double at
260 of the 419 cells, so most of the gap is the engine's accuracy, not an Excel quirk. If parity is not reached, the
forms tried and their scores go into docs/excel-semantics/statistics.md.

## Context
- Gap triage 2026-10-31e (`make eval` on origin/main 4ff408243): every failing cell below is within 15 digits of
  Excel (NumericUlp, 1 to 4 ulp), and no open task names it. T-2561 (done) added the probes for the #NUM! boundary
  and the deep tail. T-3310 owns the visibly-off cells of the same probes, plus the CHISQ.DIST.RT past-edge case.
  Phase 1b (decision 0188 R5): all cells are already at visible parity, hence `defer: phase-1b`.
- Scored against mpmath 1.4 at 50 digits (gap triage scratch script, root of the regularized upper incomplete gamma
  Q(df/2, x/2) = p in log form), Excel against the math: 0 ulp at 260 cells, 1 at 152, 2 at 6 and 5 at 1
  (large-df A230). The engine: 0 at 121, 1 at 286, 2 at 11, 3 at 1. Excel is closer at 268 cells, the engine at 126,
  and they tie at 25.
- Examples (Excel, ours, math):
  - t2561-chisq-inv-rt-num-boundary R!B1 `CHISQ.INV.RT(1E-20,4)`: 99.96639597418148, 99.9663959741815,
    99.966395974181489564.
  - t2561-chisq-inv-rt-num-large-df R!A31 `CHISQ.INV.RT(1E-150,26)`: 794.4916113085353, 794.4916113085355,
    794.49161130853545267.
  - t2561-chisq-inv-rt-num-large-df R!A230 `CHISQ.INV.RT(1E-300,45)`: 1574.5142663873226, 1574.5142663873235,
    1574.5142663873235823. The engine is on the math and Excel is 5 ulp off: this is Excel's own landing point.
- Code: `gamma_quantile` and its crossing in crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs,
  reached through `chisq_inv_of` in chisq.rs. Reuse the one gamma quantile (directive 3); CHIINV and GAMMA.INV share
  it.
- Related: T-3294 (deep lower tail at shape 20 and up), T-3311 (search region), T-3364 (extrapolated fits fall back to
  the math).
- Cells:
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (159): R!B1, B2, C4, D2, D3, D4, E1, E3, E5, F2, F3, F5, F6, F7, F8, F9, G1, G3, G5, G7, G10, H1, H3, H4, H6, H7, H8, H9, H10, H13, I1, I2, I3, I5, I6, I7, I8, I9, I10, I11, I12, J3, J5, J9, J11, J12, J13, J15, K2, K3, K4, K5, K6, K7, K9, K11, K12, K13, K15, L1, L2, L3, L4, L5, L6, L7, L9, L10, L11, L13, L15, M2, M3, M4, M9, M11, M12, M13, M14, N3, N4, N6, N7, N9, N11, N12, N13, N18, O1, O2, O3, O5, O7, O8, O9, O10, O12, O13, O15, P1, P2, P4, P5, P7, P8, P10, P11, P12, P13, P15, P18, P19, Q2, Q4, Q5, Q7, Q9, Q10, Q11, Q12, Q14, Q16, Q17, Q18, Q19, R5, R10, R11, R13, R14, R15, R16, R18, R19, S1, S2, S4, S5, S7, S8, S9, S10, S11, S13, S14, S15, S17, S19, T1, T2, T4, T5, T6, T8, T9, T12, T14, T15, T17
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (55): R!A5, A6, A14, A15, A17, A18, A21, A25, A26, A34, A43, A44, A47, A48, A49, A50, A69, A71, A76, A105, A107, A109, A134, A135, A136, A138, A139, A140, A141, A142, A143, A144, A163, A164, A165, A166, A167, A170, A171, A172, A175, A177, A178, A192, A194, A196, A199, A202, A203, A205, A206, A208, A210, A214, A216
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (101): R!A2, A10, A11, A12, A13, A19, A20, A21, A37, A39, A40, A44, A47, A49, A55, A67, A71, A74, A76, A98, A100, A102, A103, A105, A108, A121, A122, A123, A128, A130, A132, A133, A135, A137, A139, A145, A170, A172, A175, A176, A177, A193, A194, A197, A198, A199, A200, A202, A203, A204, A205, A206, A208, A218, A220, A221, A222, A224, A227, A228, A229, A236, A238, A239, A240, A242, A265, A267, A268, A271, A272, A274, A279, A290, A291, A293, A294, A296, A297, A298, A300, A301, A303, A304, A306, A308, A309, A310, A311, A316, A317, A319, A337, A338, A343, A345, A346, A347, A348, A349, A350
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (104): R!A21, A31, A51, A52, A53, A62, A65, A71, A72, A81, A82, A83, A84, A91, A93, A94, A95, A101, A102, A103, A104, A113, A114, A116, A122, A123, A124, A125, A126, A131, A132, A133, A134, A135, A136, A143, A144, A145, A146, A151, A152, A158, A163, A165, A167, A168, A173, A175, A176, A177, A181, A183, A185, A188, A189, A192, A198, A199, A202, A203, A204, A205, A206, A207, A208, A209, A213, A214, A215, A216, A217, A218, A219, A220, A221, A222, A225, A228, A229, A230, A232, A234, A235, A240, A243, A245, A246, A247, A248, A249, A251, A252, A253, A258, A259, A261, A264, A267, A270, A271, A272, A273, A276, A278

## Acceptance criteria
- [ ] The 419 known-failure lines above disappear from eval/baseline/probes-numeric.txt with no new known-failure line
      in any baseline, or the forms tried and their per-case scores are in statistics.md and the rest move to a
      follow-up.
- [ ] A more accurate crossing (closer to the correctly rounded quantile) is not a fit and needs no R4 guards. Any
      Excel-specific form fitted to Excel's bits follows decision 0188 R4 (reviewer-excel-fidelity check 7), recorded
      in the Outcome:
  - hold-out counts: the fit-lab split (about 30% by cell hash), train and test exact counts side by side, and the
    form beats the correctly rounded candidate on test;
  - a fresh log-uniform sweep, filled after the form is frozen, and its result;
  - the Gate A envelope result on that sweep (points within 15 digits of Excel, within 1E-9 of the math, outside
    both, error against number);
  - every fitted constant or cut-point tagged `/// FITTED(T-3432, <n> cells, <range>)` and applied only inside the
    measured p and df range;
  - no fitting below the oracle's reproducibility floor (decision 0112: up to 4 ulp between web and desktop): the 418
    cells where Excel is 2 ulp or less off the math are not fit targets for an Excel-specific form;
  - no branch whose only purpose is to keep an already-matching cell matching.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of these probes and of t2561-chisq-dist-rt-past-edge (T-3310).
- CHISQ.INV and GAMMA.INV deep lower tail (T-3294) and the search region (T-3311).
