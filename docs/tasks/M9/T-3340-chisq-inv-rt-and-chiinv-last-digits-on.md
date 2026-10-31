---
id: T-3340
title: "CHISQ.INV.RT and CHIINV last digits on the T-2561 far-tail edge probes and m9-legacy-chiinv (294 cells, 1 to 10 ulp)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3310, T-3146]
defer: phase-1b
---

## Goal
The CHISQ.INV.RT and CHIINV cells below, all already within 15 significant digits of Excel, match Excel to the bit.

## Context
Gap triage 2026-10-31 (`make eval` on this branch's main base, excel-graph evidence). These failing cells have no open
owner: T-2561 (done) added the far-tail edge probes and owned only their type and error class, and T-2860 and T-1047
(done) left the m9-legacy-chiinv last digits. No cell here is visibly off (decision 0083), so this is Phase 1b work
(decision 0114) and stays deferred.

Two groups, one kernel (the right-tail gamma quantile, `incomplete_gamma::gamma_quantile` behind
crates/rustsheet-engine/src/functions/statistical/chisq.rs `chisq_inv_of`):
- **Far right tail, df 3 to 42, p = 1E-20 to 1E-300** (t2561 probes): 252 of 260 cells are 1 ulp off, 8 are 2 ulp.
  Excel vs ours: probes-numeric/t2561-chisq-inv-rt-num-boundary R!B1 `CHISQ.INV.RT(1E-20,4)` 99.96639597418148 vs
  99.9663959741815; t2561-chisq-inv-rt-num-large-df-edge R!A10 `CHISQ.INV.RT(1E-151,24)` 792.0240556387595 vs
  792.0240556387596; R!A100 `CHISQ.INV.RT(1E-204,32)` 1072.2436632869228 vs 1072.2436632869226. Both signs occur,
  so this is not a one-sided stopping bias; fit against Excel's own CHISQ.DIST.RT at the neighbouring doubles
  (t2561-chisq-dist-rt-past-edge has the cdf side) before choosing a crossing convention.
- **m9-legacy-chiinv moderate p** (CHIINV and CHISQ.INV.RT agree cell for cell): 1 or 2 ulp except df 1, where
  `CHIINV(0.05,1)` A31/A32 is 3.8414588206941236 vs 3.841458820694128 (10 ulp) and `CHIINV(0.5,1)` A41/A42 is
  0.4549364231195729 vs 0.4549364231195725 (7 ulp). At df 1 the engine takes the lower quantile of 1 - p
  (statistics.md, "CHISQ.INV.RT and CHIINV quirks"); a 100-digit check (gap triage scratch, mpmath erfinv) puts the
  exact lower quantile of the double 1 - p 2 ulp from Excel at both p, so the engine's df-1 path itself loses
  5 to 8 ulp there.

Not here: the p = 0.999 cells of m9-legacy-chiinv (A65, A66, A69, A70) and the visibly-off `CHIINV(1E-9,1)` pair
(A1, A2), which are Excel's landing point in the flat band of its double cdf (T-3146); the visibly-off far-tail
CHISQ.DIST.RT cells and the 3 quantiles of t2561-chisq-inv-rt-num-edge and -num-large-df (T-3310). Re-run the cell
list after T-3310 and T-3146 land: both change this kernel.

Cells (probes-numeric is ungated, so these are the failing cells of the run, not baseline lines):
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (159): R!B1, B2, C4, D2, D3, D4, E1, E3, E5, F2, F3, F5, F6, F7, F8, F9, G1, G3, G5, G7, G10, H1, H3, H4, H6, H7, H8, H9, H10, H13, I1, I2, I3, I5, I6, I7, I8, I9, I10, I11, I12, J3, J5, J9, J11, J12, J13, J15, K2, K3, K4, K5, K6, K7, K9, K11, K12, K13, K15, L1, L2, L3, L4, L5, L6, L7, L9, L10, L11, L13, L15, M2, M3, M4, M9, M11, M12, M13, M14, N3, N4, N6, N7, N9, N11, N12, N13, N18, O1, O2, O3, O5, O7, O8, O9, O10, O12, O13, O15, P1, P2, P4, P5, P7, P8, P10, P11, P12, P13, P15, P18, P19, Q2, Q4, Q5, Q7, Q9, Q10, Q11, Q12, Q14, Q16, Q17, Q18, Q19, R5, R10, R11, R13, R14, R15, R16, R18, R19, S1, S2, S4, S5, S7, S8, S9, S10, S11, S13, S14, S15, S17, S19, T1, T2, T4, T5, T6, T8, T9, T12, T14, T15, T17
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (101): R!A2, A10, A11, A12, A13, A19, A20, A21, A37, A39, A40, A44, A47, A49, A55, A67, A71, A74, A76, A98, A100, A102, A103, A105, A108, A121, A122, A123, A128, A130, A132, A133, A135, A137, A139, A145, A170, A172, A175, A176, A177, A193, A194, A197, A198, A199, A200, A202, A203, A204, A205, A206, A208, A218, A220, A221, A222, A224, A227, A228, A229, A236, A238, A239, A240, A242, A265, A267, A268, A271, A272, A274, A279, A290, A291, A293, A294, A296, A297, A298, A300, A301, A303, A304, A306, A308, A309, A310, A311, A316, A317, A319, A337, A338, A343, A345, A346, A347, A348, A349, A350
- probes-numeric/m9-legacy-chiinv.case.json (34): Calc!A7, A8, A11, A12, A15, A16, A17, A18, A19, A20, A21, A22, A25, A26, A31, A32, A35, A36, A39, A40, A41, A42, A45, A46, A51, A52, A55, A56, A57, A58, A61, A62, A85, A86

Semantics: docs/excel-semantics/statistics.md, the CHISQ.INV.RT and CHIINV quirks bullet and the T-2561 deep-tail
entries. Fit with the fit lab (decision 0056: two attempts, then time-box).

## Acceptance criteria
- [ ] The 294 cells above are bit-exact in `make eval` (or the ones the fitted rule cannot reach are named in the
      Outcome with their ulps), with no new known-failure line in any eval/baseline file (`make eval-baseline`).
- [ ] The rule found, or the candidates tried with their bit-exact counts on these three cases, are in
      docs/excel-semantics/statistics.md.
- [ ] Iteration stays bounded on untrusted input (the shared term budget; p next to 0 or 1; df up to 1E10).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off cells (T-3146, T-3310) and the `#NUM!` edge table (`RIGHT_TAIL_P_MIN`, done in T-2561).
- The gen-m9 CHIINV last digits (T-2695) and the lower-tail gamma quantile digits (T-3239).
