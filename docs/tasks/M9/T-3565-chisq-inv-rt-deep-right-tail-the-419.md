---
id: T-3565
title: "CHISQ.INV.RT deep right tail: the 419 T-2561 edge-probe values 1 to 4 ulp off Excel (Excel is the correctly rounded root on most)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3310]
defer: phase-1b
---

## Goal
CHISQ.INV.RT's value at p from 1E-20 to 1E-300 and df 3 to 50 (the region below Excel's `#NUM!` edge where it is
still a number) matches Excel to the bit on the four T-2561 edge probes, or within 1 ulp stated per cell. Today 419
of their 1,320 cells are 1 to 4 ulp off (386 by 1, 32 by 2, one by 4) and no task owns them.

## Context
- Gap triage 2026-11-01 (`make eval` on b98b3c00d). All four cases are excel-graph, from T-2561 (statistics.md
  "CHISQ.INV.RT and CHIINV quirks", the `#NUM!` edge bullet). Since T-2561 every cell has Excel's type, error and
  value class. What is left is 419 cells within 4 ulp of Excel, none visibly off. T-3310 owns the three other failing
  quantile cells of these cases (num-edge A72, num-large-df A210, A280, each more than 4 ulp off).
- Examples (Excel vs ours):
  - t2561-chisq-inv-rt-num-boundary B1 `CHISQ.INV.RT(1E-20,4)`: 99.96639597418148 vs 99.9663959741815;
  - t2561-chisq-inv-rt-num-edge A105 `CHISQ.INV.RT(1E-121,19)`: 631.7419795466809 vs 631.7419795466808;
  - t2561-chisq-inv-rt-num-large-df A101 `CHISQ.INV.RT(1E-150,33)`: 818.750619301226 vs 818.7506193012262.
- Against 50-digit math (mpmath root of Q(df/2, x/2) = p, 80 failing cells drawn at random, this triage): Excel is
  the correctly rounded root at 52 and ours at 24, and neither at 4. The largest relative error is 2.7E-16 for
  Excel and 1.9E-16 for ours. So most of the gap is where our search lands: it takes a neighbour of the correctly
  rounded root. A correctly rounded right-tail quantile would match Excel on about two thirds of these cells.
  First measure what it costs on the 898 cells that pass today and on gated cells (gen-m9, ironcalc-calc), and on
  the far-tail gamma quantiles of T-3294 and T-3311.
- The quantile sees Excel's tail Q, which T-3310 finds up to 5.6E-14 relative off ours in this x range, though a Q
  error that size moves the root by about 1E-16 relative. So this waits for T-3310 and measures after it.
- Code: `chisq_inv_of` in crates/rustsheet-engine/src/functions/statistical/chisq.rs (`RIGHT_TAIL_P_MIN`), and
  `gamma_quantile` in crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs.
- T-3363 (Excel's own inverse algorithm for the Student t, F and chi-squared quantiles) is related. If it lands
  first, re-measure these cells on it before fitting anything.

Cells (every failing value cell of the four cases except T-3310's three):
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (159): R!B1:B2, C4, D2:D4, E1, E3, E5, F2:F3, F5:F9, G1, G3,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): G5, G7, G10, H1, H3:H4, H6:H10, H13, I1:I3, I5:I12,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): J3, J5, J9, J11:J13, J15, K2:K7, K9, K11:K13, K15,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): L1:L7, L9:L11, L13, L15, M2:M4, M9, M11:M14, N3:N4,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): N6:N7, N9, N11:N13, N18, O1:O3, O5, O7:O10,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): O12:O13, O15, P1:P2, P4:P5, P7:P8, P10:P13, P15,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): P18:P19, Q2, Q4:Q5, Q7, Q9:Q12, Q14, Q16:Q19, R5,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): R10:R11, R13:R16, R18:R19, S1:S2, S4:S5, S7:S11,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): S13:S15, S17, S19, T1:T2, T4:T6, T8:T9, T12,
- probes-numeric/t2561-chisq-inv-rt-num-boundary.case.json (cont.): T14:T15, T17
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (55): R!A5:A6, A14:A15, A17:A18, A21, A25:A26, A34, A43:A44,
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (cont.): A47:A50, A69, A71, A76, A105, A107, A109, A134:A136,
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (cont.): A138:A144, A163:A167, A170:A172, A175, A177:A178, A192,
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (cont.): A194, A196, A199, A202:A203, A205:A206, A208, A210,
- probes-numeric/t2561-chisq-inv-rt-num-edge.case.json (cont.): A214, A216
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (104): R!A21, A31, A51:A53, A62, A65, A71:A72, A81:A84, A91,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A93:A95, A101:A104, A113:A114, A116, A122:A126,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A131:A136, A143:A146, A151:A152, A158, A163, A165,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A167:A168, A173, A175:A177, A181, A183, A185,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A188:A189, A192, A198:A199, A202:A209, A213:A222,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A225, A228:A230, A232, A234:A235, A240, A243,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A245:A249, A251:A253, A258:A259, A261, A264, A267,
- probes-numeric/t2561-chisq-inv-rt-num-large-df.case.json (cont.): A270:A273, A276, A278
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (101): R!A2, A10:A13, A19:A21, A37, A39:A40, A44, A47,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A49, A55, A67, A71, A74, A76, A98, A100,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A102:A103, A105, A108, A121:A123, A128, A130,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A132:A133, A135, A137, A139, A145, A170, A172,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A175:A177, A193:A194, A197:A200, A202:A206,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A208, A218, A220:A222, A224, A227:A229, A236,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A238:A240, A242, A265, A267:A268, A271:A272,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A274, A279, A290:A291, A293:A294, A296:A298,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A300:A301, A303:A304, A306, A308:A311,
- probes-numeric/t2561-chisq-inv-rt-num-large-df-edge.case.json (cont.): A316:A317, A319, A337:A338, A343, A345:A350

## Acceptance criteria
- [ ] The Outcome gives per-case counts (bit-exact, within 1 ulp, within 4 ulp) for main, for the chosen change, and
      for a correctly rounded right-tail quantile, on these 419 cells and on the cells of the four cases that pass
      today.
- [ ] Any landing rule picked by scoring Excel cells carries the decision 0188 R4 guards (reviewer-excel-fidelity check
      7): a hold-out split fixed before scoring (by case or by cell hash, about 30%), with its margin; train and
      hold-out counts for the chosen and the correctly rounded candidate; a fresh probes-numeric sweep after the
      freeze (p mantissas inside a decade, df 3 to 60); the Gate A envelope result on that sweep; a
      `FITTED(T-3565, n_cells, range)` tag and range, with accurate math outside it; nothing fitted below the
      reproducibility floor; and no keep-matched branch.
- [ ] No cell of any suite goes from within 15 digits to visibly off; no gated bit-exact cell is lost without a
      decision 0082 record.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` edge itself (T-2561, done), CHISQ.DIST.RT's far tail (T-3310), the lower-tail quantiles (T-3294,
  T-3311), df 1 and 2 (T-3336, T-3337).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
