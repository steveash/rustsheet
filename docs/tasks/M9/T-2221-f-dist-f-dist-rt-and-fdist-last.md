---
id: T-2221
title: "F.DIST, F.DIST.RT and FDIST last digits: fit Excel's closed forms, complement and tail choice on the sweep"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: [T-2219, T-2220]
---

## Goal
F.DIST (cdf and density), F.DIST.RT and FDIST give Excel's double bit for bit on the F-distribution
probes, including where Excel is not correctly rounded. Needs an oracle sweep: T-2220 provides it (`probes-numeric/t2220-f-dist-sweep-*.case.json`, findings in statistics.md "Dense F sweep (T-2220)").

## Context
Gap triage 2026-10-21 (`make eval` on `main` fce6020d). After T-0970, 107 F-distribution cells
are still not bit-exact and no open task owns them. They exclude T-2219's 16 accuracy cells and the
cells owned by T-1849 (booleans), T-2095 (x = 0) and T-1538 (df of 1E7 and above):
- ironcalc-calc/STATISTICAL/F_DIST.xlsx Sheet1!F3 (`F.DIST(0.7,4,2.4,FALSE)`, Excel
  0.40509259259259256, ours 1 ulp above). This is a known-failure line in eval/baseline/ironcalc-calc.txt.
- probes-numeric/m9-f-dist-rt A1-A4, A13-A16, A19, A20.
- probes-numeric/m9-f-dist A1-A6, A8, A10, A16, A18, A19, A21, A22, A24.
- probes-numeric/m9-legacy-fdist A21, A22, A25, A26, A29-A32, A35, A36, A41, A42, A45-A52,
  A55-A62, A65-A72, A75-A80.
- probes-numeric/t1049-fdist A1-A3, A5, A7-A10, A12-A14, A16-A18, A20, A22-A24, A32-A35, A37, A38, A43.
- probes-numeric/t1049-fdistrt A1, A2, A4, A5, A7-A12, A17, A18, A23-A25.

Most are 1-4 ulp. Six are visibly off (more than half a unit in the 15th digit) because Excel itself is
off the true value by more than that, so no accurate kernel reaches them. A 50-digit mpmath
reference, run in the 2026-10-21 triage, gives these relative errors:
- m9-f-dist-rt A16 and m9-legacy-fdist A55, A56: `F.DIST.RT(10,10,10)` = 0.0005715525434020334,
  Excel +1.4E-15, ours -5.4E-16.
- m9-f-dist-rt A20: `F.DIST.RT(10,50,100)` = 3.009600356083383E-22, Excel -4.1E-15, ours -2.3E-15.
- m9-legacy-fdist A47, A48: `FDIST(3.5,5,50)` = 0.008642212995925193, Excel +7.1E-16, ours -1.7E-15.
- t1049-fdist A16: `F.DIST(2.5,3,7,FALSE)` = 0.09478619896436714, Excel +9.5E-16, ours -2.2E-16.

Excel's d1 = d2 = 1 values are not the exact ones either: `F.DIST(1,1,1,TRUE)` = 0.5000000000000001,
`F.DIST.RT(1,1,1)` = 0.4999999999999999, and `F.DIST(0.5,1,1,TRUE)` = 0.39182655203060723, 1 ulp below
`2/PI()*ATAN(SQRT(0.5))` in doubles. statistics.md's T-0970 section has the arctangent lead and the
two PARTIAL rules (exact complement, one-evaluation tails) that the oracle contradicts at
t1049-fdistrt A2, A8. Fit lab precedent: T-1744 (`uv run rustsheet-fitlab`) and
docs/research/excel-kernel-playbook.md. Code: `functions/statistical/incomplete_beta.rs` and the F
callers in `functions/statistical/`.

## Acceptance criteria
- [ ] The F_DIST.xlsx Sheet1!F3 line disappears from eval/baseline/ironcalc-calc.txt
      (`make eval-baseline`), with no new line in any baseline.
- [ ] Of the 106 probes-numeric cells listed above, at least 90% are bit-exact, and every one is within
      half a unit of the 15th significant digit. The ones that are not bit-exact are listed in the
      Outcome with their ulp distance.
- [ ] Each T-2220 sweep cell of F.DIST, F.DIST.RT and FDIST is bit-exact or within 4 ulp. The rate is
      reported in the Outcome.
- [ ] statistics.md's T-0970 section replaces the PARTIAL rules with the fitted formulation (VERIFIED on
      the sweep) or records which hypotheses were ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy fix at large a + b (T-2219, a dependency). F.INV and F.INV.RT (T-0974, which should
  invert whatever cdf this lands). BETA.DIST's cdf (T-2119). df of 1E7 and above (T-1538, T-1864).

## Visible-parity stage
Decision 0083: this stage is met when the six visibly-off cells above (m9-f-dist-rt A16, A20;
m9-legacy-fdist A47, A48, A55, A56; t1049-fdist A16) are within half a unit of the 15th
significant digit of Excel's value, and no other cell this task owns is newly visibly off. Bit-level
criteria still unmet at that point move to a follow-up task without the campaign key.
