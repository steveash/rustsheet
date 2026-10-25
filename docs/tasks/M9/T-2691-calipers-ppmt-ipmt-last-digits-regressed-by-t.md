---
id: T-2691
title: "Calipers PPMT/IPMT last digits regressed by T-2490 (25 cells in pv23_007/008/009/056)"
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
