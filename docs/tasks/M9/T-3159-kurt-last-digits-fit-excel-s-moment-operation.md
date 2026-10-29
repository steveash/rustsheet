---
id: T-3159
title: "KURT last digits: fit Excel's moment operation order on 48 gen-m9 cells (7.000000000000002 where ours is 7)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
KURT follows Excel's own operation order for the sample excess kurtosis, so the 48 failing gen-m9 KURT cells match
Excel, the three visibly-off ones first.

## Context
- Gap triage 2026-10-29 (opus refill, report on 5d089378). No open task names these cells. T-1754 (deferred) owns
  three other KURT last-digit cells (probes/stats-evidence-floor-t1623 Q36, ironcalc-calc
  MEADIAN_KURT_SKEW_HARMEAN.xlsx F2, probes/stat-argument-modes H13) and says "fit all three cells with one order";
  they are the same root cause, so fit all 51 cells together and close T-1754 as covered in this task's Outcome.
- Visibly off (Excel vs ours):
  - gen-m9/27601819397dd56e Calc!A200 `KURT(Data!E4:E11)`: 3.9999999999979536 vs 3.9999999999979607.
  - gen-m9/7bd219ec0c0c5e22 Calc!A279 `KURT(Data!A1:A12)`: -0.7628272385808703 vs -0.7628272385808685.
  - gen-m9/c2974c061384fdd9 Calc!A55 `KURT(Data!A6:A6,Data!D1:D11)`: 0.734384233823282 vs 0.7343842338232829.
- A recurring shape, likely a quirk of Excel's formula: where the exact result is 7 (n = 8 points, one outlier),
  Excel gives 7.000000000000002 and we give 7.0: gen-m9/05d44f6dd6cfc567 A185, gen-m9/06b4e8f5de409d5e A14,
  gen-m9/3e98b26cf6713e77 A359, gen-m9/4ab2ba98d26c8a5e A356, gen-m9/68d8ffb69ff739d8 A25, gen-m9/abc2939ebd1e2365 A34,
  gen-m9/c18dbf21f11093f3 A307, gen-m9/ee2497a5e683c5d9 A350; but gen-m9/d8266bc15533702a A46 is Excel 7.0, ours
  6.999999999999998, and `KURT({1,..,10})` is Excel -1.2000000000000002, ours ...006 (gen-m9/a013a45a8fbcab4f A2,
  gen-m9/a54c738c697617c3 A41). These pin the order of the n(n+1)/((n-1)(n-2)(n-3)) factor, the Σz⁴ sum and the
  3(n-1)²/((n-2)(n-3)) term.
- All failing cells: `make eval-show CASE=gen-m9/<id>.case.json` for the cases above and 05d44f6dd6cfc567 A246,
  0a614cfe974b59c3 A139 A322, 0a9226016e8f5f33 A88, 27601819397dd56e A108, 285d87cea1aaa73c A181, 2a18b2f5a4dc43ac A246,
  320d51765af9b6b7 A74, 40425341c5034eed A313, 4250f880ab640c49 A45, 58e3253bbd3a7a82 A314, 68d8ffb69ff739d8 A330,
  7c20351562a2d71d A134 A226 A272, 8dd2312c96effb1b A277, 9cca17438a7e6f8a A139 A261, a61e547a1cab994c A75,
  a790a4d4a6a82756 A140, b6593d7dd13f2dac A363, bbdfc06cae0e0d71 A123 A353, bc40d37cb4d8301d A308, c18dbf21f11093f3
  A77 A215, c468f5046aadfbbc A267, c6cfda6bd10cae93 A24, caa1eed48b1883fe A284, cde551ea787f713a A365,
  e6cf3b20a014e6a3 A377, f34f66fb7f81e673 A345, fec39efaba9284c3 A38 A222.
- Numeric fitting with 51 Excel points and many ties: if the candidate orders (mean then deviations; Σx, Σx², Σx³,
  Σx⁴ raw moments; standardized z with sample s; two-pass with a correction) cannot be told apart on these, file a
  research-lane `excel-probe` sweep (small n, exact-7 and exact-integer kurtosis shapes) and make this task depend
  on it ("Needs an oracle sweep"). DEVSQ's order (T-0951, T-1749) may share the deviation sum: reuse what they
  find, never a second copy.
- Code: KURT in crates/rustsheet-engine/src/functions/statistical/ (`rg -n 'fn fn_kurt'`).

## Acceptance criteria
- [ ] These 48 gen-m9 KURT known-failure lines disappear from eval/baseline/gen-m9.txt, and T-1754's three lines
      from eval/baseline/probes.txt and ironcalc-calc.txt, with no new known-failure lines (`make eval-baseline`);
      at minimum the three visibly-off cells, with the remaining last-digit cells moved to a `defer: phase-1b`
      follow-up if the order is time-boxed (decision 0056).
- [ ] The order (or the candidates scored and their counts) is recorded in docs/excel-semantics/statistics.md,
      citing the exact-7 cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SKEW and SKEW.P (same moments, separate cells): note in the Outcome whether the order carries over.

## Visible-parity stage
Decision 0083 and decision 0114: done when the three visibly-off cells above are within half a unit of the 15th
significant digit of Excel's value. Bit-exact criteria still unmet then move to a `defer: phase-1b` follow-up named in
the Outcome. Stage `visible`: the exact results (7, -1.2) show Excel is not correctly rounded, so only its
operation order gives its digits.
