---
id: T-2767
title: "F.INV, F.INV.RT and FINV last digits on 28 gen-m9 quantile cells no task lists (1 to 5 ulp)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2500, T-2511]
defer: phase-1b
---

## Goal
The 28 F quantile cells of the gen-m9 sweep listed below, all already within 15 significant digits of Excel, match
Excel to the bit.

## Context
Gap triage 2026-10-26 (`make eval` on 1c8b3d34, excel-graph gen-m9 evidence). No open task lists these cells: T-2500
owns the F quantile cells a unit or two off the 15th digit (and, since this triage, the three gen-m9 cells that are
visibly off, which are not listed here), and T-2511 re-bases BETA.INV, F.INV, F.INV.RT and T.INV on the ported
incomplete beta. Every cell here is 1 to 5 ulp off and not visibly off, so this is Phase 1b work (decision 0114) and stays
deferred. It depends on T-2500 and T-2511 because both change the crossing these cells sit on: re-run the list after
them before any fit.

Root cause (suspected): each value is the crossing of a cdf that is itself a few ulp from Excel's F.DIST/F.DIST.RT, or
Excel's quantile search stops at a different iterate (T-1745 found BETA.INV is a crossing of Excel's own cdf;
statistics.md). The engine's result is usually the correctly rounded root, as in T-2500's measured cells, so this is
fitting Excel's search and kernel, not accuracy work. Numeric fitting, so opus. If the cached cells cannot separate
candidate stopping rules, file an oracle sweep (research lane) of F.INV/F.INV.RT at these shapes over a dense p grid
first and depend on it.

Examples (`make eval-show CASE=gen-m9/<id>.case.json`), Excel vs ours:
- gen-m9/40425341c5034eed Calc!A229 `=F.INV(0.05,10,4)`: 0.28751745630752723 vs 0.287517456307527 (4 ulp)
- gen-m9/8f2ac25945af87da Calc!A283 `=F.INV.RT(0.001,7,9)`: 10.697947908142359 vs 10.697947908142368 (5 ulp)
- gen-m9/85f48d5cdfb24555 Calc!A214 `=F.INV.RT(0.975,10000000000.5,5)`: 0.38963563766471687 vs 0.3896356376647166 (5 ulp)
- gen-m9/899f9555e67da2d6 Calc!A371 `=FINV(1E-10,8,(Data!A$9+123456789.123))`: 7.924747393225468 vs 7.924747393225465 (4 ulp)
- gen-m9/8dd2312c96effb1b Calc!A357 `=FINV(Data!A6,4,30)`: 2.1422348562884994 vs 2.142234856288499 (1 ulp)
- gen-m9/fec39efaba9284c3 Calc!A106 `=FINV(0.95,9,9)` and gen-m9/bbdfc06cae0e0d71 Calc!A339 `=F.INV.RT(0.95,9,9)`: both
  0.3145749061513079 vs 0.314574906151308 (2 ulp), so FINV and F.INV.RT share the path.
Code: crates/rustsheet-engine/src/functions/statistical/ (the F quantile search over `incomplete_beta.rs`).

Owned gen-m9 cells (known-failure lines in eval/baseline/gen-m9.txt), each with F.INV, F.INV.RT or FINV as the outer call:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A192
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A218
- gen-m9/40425341c5034eed.case.json (2): Calc!A229, A367
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A120
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A214
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A371
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A283, A357
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A283
- gen-m9/a5828d3925575713.case.json (1): Calc!A328
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A155
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A339
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A167
- gen-m9/d8266bc15533702a.case.json (1): Calc!A39
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A204
- gen-m9/eced4bb6a0234c9e.case.json (3): Calc!A18, A340, A381
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A29, A75
- gen-m9/f34f66fb7f81e673.case.json (3): Calc!A19, A117, A314
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A101
- gen-m9/fec39efaba9284c3.case.json (3): Calc!A30, A106, A244

## Acceptance criteria
- [ ] These 28 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`) with no new ones in any
      baseline, or the Outcome records the models tried and their bit-exact counts (decision 0056 time box) and the
      cells still open move to a follow-up.
- [ ] The F quantile model (search, stopping rule, kernel calls) is recorded in docs/excel-semantics/statistics.md with
      these cells as evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off F quantile cells (T-2500) and the kernel port itself (T-2504 to T-2511).
- BETA.INV (T-2119, T-2555) and T.INV (T-0967).
