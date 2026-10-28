---
id: T-2708
title: "Calipers PPMT/IPMT last digits regressed by T-2490 (35 cells in pv23_007/008/009/056), plus one CHISQ.INV cell"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The calipers PPMT and IPMT cells that passed at the calipers baseline (#1706) pass again, or each is explained as a
change that moves the engine closer to Excel elsewhere, with the calipers baseline ratcheted afterwards.

## Context
Found by T-0462. Calipers is not in `make check` or CI (suite.toml): it is gated by
`rustsheet-eval check --suite calipers`. On main at 2c34755d that check reports 36 regressions in cases whose checked
count did not change. The 25 it prints are NumericUlp cells in calipers/pv_goldens/pv23_007 (11), pv23_008 (4),
pv23_009 (1) and pv23_056 (9). They are PPMT cells such as `=PPMT($B$19/12,K3,$B$20*12,$B$17*$B$2,0,)` and
`=IF(AH3<=Summary!$C$37,0,PPMT(Summary!$C$36/12,AH3,Summary!$C$38*12,Summary!$C$32,0,))`, plus SUMIFs over such
columns. pv23_056 AH27: Excel -93094.88005032795, ours -93094.88005032793.
Likely cause: T-2490 (#1763, "IPMT cancels on fv, and IPMT/PPMT keep values past (1+rate)^nper's range"), the first
PPMT/IPMT change after the calipers baseline (#1706). T-2242 (#1680) came before the baseline. Confirm by running the
check on the commit before #1763.
Code: functions/financial/annuity.rs (PPMT/IPMT).
The full list (36 cells, from a full `rustsheet-eval run --suite calipers` compared with eval/baseline/calipers.txt:
failing cells of cases whose checked count is unchanged that are not known failures; `check` prints only 25):
- calipers/pv_goldens/pv23_007/golden.xlsx (11): Pro Forma Cash Flow Statement!AF13, Pro Forma Cash Flow Statement!AV13, Pro Forma Cash Flow Statement!BG15, Pro Forma Cash Flow Statement!BI13, Pro Forma Cash Flow Statement!BI15, Pro Forma Cash Flow Statement!BO13, Pro Forma Cash Flow Statement!G15, Pro Forma Cash Flow Statement!G24, Pro Forma Cash Flow Statement!N13, Pro Forma Cash Flow Statement!N15, Pro Forma Cash Flow Statement!N24
- calipers/pv_goldens/pv23_008/golden.xlsx (4): Debt Financing!E231, Est Property Value!B47, Est Property Value!C47, IRR!B26
- calipers/pv_goldens/pv23_009/golden.xlsx (1): Equity Waterfall Model!T14
- calipers/pv_goldens/pv23_056/golden.xlsx (19): Monthly Cash Flow Statement!AE27, Monthly Cash Flow Statement!AH27, Monthly Cash Flow Statement!AK27, Monthly Cash Flow Statement!AQ27, Monthly Cash Flow Statement!AR27, Monthly Cash Flow Statement!AV27, Monthly Cash Flow Statement!AY27, Monthly Cash Flow Statement!BB27, Monthly Cash Flow Statement!BE27, Monthly Cash Flow Statement!BI27, Monthly Cash Flow Statement!BI29, Monthly Cash Flow Statement!BK27, Monthly Cash Flow Statement!BM27, Monthly Cash Flow Statement!BM29, Monthly Cash Flow Statement!BR27, Monthly Cash Flow Statement!BS27, Monthly Cash Flow Statement!BT27, Monthly Cash Flow Statement!BT29, Monthly Cash Flow Statement!BX29
- calipers/roundtrip/formula_stress_test/golden.xlsx (1): Statistical!B13
The roundtrip/formula_stress_test cell is `_xlfn.CHISQ.INV(B11,D11)` (NumericUlp), a different function. Find the
commit that changed it (git log on functions/statistical/chisq.rs) and fix it in this task or split it out.
Until this is fixed, the calipers baseline cannot be ratcheted: `update-baseline --suite calipers` refuses
regressions. That blocks recording fixes such as T-0462's 492 add-in cells and the about 600 other cells fixed on
main since #1706.

## Acceptance criteria
- [ ] Every calipers regression the check reports is listed here and either passes again or is shown to be the cost of
      a change that matches more Excel cells (with the evidence). Then `rustsheet-eval update-baseline --suite calipers`
      is run and committed.
- [ ] No gated baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making calipers part of `make check`.
