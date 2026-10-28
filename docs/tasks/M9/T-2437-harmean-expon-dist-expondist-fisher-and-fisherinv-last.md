---
id: T-2437
title: "HARMEAN, EXPON.DIST, EXPONDIST, FISHER and FISHERINV last digits on the gen-m9 sweep (22 cells)"
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
HARMEAN, EXPON.DIST, EXPONDIST, FISHER and FISHERINV give Excel's last digits on the gen-m9 sweep cells below, from a fitted
evaluation order, or the task records the candidates tried (decision 0056).

## Context
Gap triage of the T-2255 gen-m9 sweep. These 22 cells are within half a unit of the 15th significant digit (not visibly off,
so no campaign key) and no open accuracy task covers these functions. HARMEAN (17): n / sum(1/x) in some order, e.g.
`HARMEAN(Data!D6:D10,Data!C6:C9)` Excel 0.569594502522996, ours 0.5695945025229961; EXPON.DIST/EXPONDIST (3): 1 - exp(-lambda x)
or its complement; FISHER/FISHERINV (2): statistics.md "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1), with its cancellation near 0"
has the inverse's form, FISHER's (0.5*LN((1+x)/(1-x)) in doubles) is the candidate to test (visibly-off FISHER cells
near 0 are T-2436's). Fit in the fit lab (tools, `rustsheet-fitlab`) on these cells plus the passing ones of the same
functions in eval/ (a candidate must not break a passing cell).
Code: functions/statistical/ (harmean in count_and_average.rs or its own file; expon; fisher).

Owned gen-m9 cells, last digits only (T-2255 triage, excel-graph, 2026-10-22): 22 known-failure lines in eval/baseline/gen-m9.txt, 22 formulas.
Examples:
- gen-m9/06b4e8f5de409d5e Calc!A140 `=EXPONDIST((TAN(100*PI())+0),1E+8,"TRUE")`: Excel 1.959369974159244e-07, ours 1.9593699741592436e-07
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A140, A278
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A298
- gen-m9/530c636749a85209.case.json (1): Calc!A251
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A324
- gen-m9/623663cacea53bbc.case.json (2): Calc!A207, A318
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A41
- gen-m9/7674c89c824aca43.case.json (1): Calc!A268
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A44
- gen-m9/a54c738c697617c3.case.json (1): Calc!A252
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A191
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A306
- gen-m9/b4173fe25109a238.case.json (1): Calc!A143
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A42
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A365
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A234
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A230, A352
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A382
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A30
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A166

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `FISHER(1E-15/3.14159)` is 2.2204460492503126E-16 in Excel (ours 3.3306690738754686E-16): visibly off, not a last digit.
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/4ab2ba98d26c8a5e Calc!A395 `=FISHER((Data!E4/Data!$D2))`: Excel 2.2204460492503126e-16, ours 3.3306690738754686e-16
Cells:
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A395

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Either the fitted order is implemented and these lines leave eval/baseline/gen-m9.txt (`make eval-baseline`) with no
      new known-failure line in any baseline, or the task is `blocked` with "Blocked because: time-boxed (decision 0056)"
      and the candidates' bit-exact counts in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off cells of these functions (other tasks).

## Visible-parity stage
Decision 0083/0114: this task is done when every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move to a follow-up task filed without the key and with `defer: phase-1b`.
Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: gen-m9/4ab2ba98d26c8a5e Calc!A395 `=FISHER((Data!E4/Data!$D2))`: Excel 2.2204460492503126e-16, ours 3.3306690738754686e-16.
