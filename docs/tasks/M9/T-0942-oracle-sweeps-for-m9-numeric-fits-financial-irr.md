---
id: T-0942
title: "Oracle sweeps for M9 numeric fits (financial, IRR/RATE, trig reduction, Bessel, inverse hyperbolics, inverse normal)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Oracle-filled probe cases give each blocked M9 numeric-fitting task enough Excel points to fit its formulation, instead of fitting blind to the few values cached in Excel-authored workbooks.

## Context
T-0018: M9 last-digit tasks were being attempted with only cached workbook values. T-0901 and
T-0925 were blocked, and T-0916 was found not fittable without the oracle. The oracle exists now
(decision 0013), and `rustsheet-gen --policy numeric-rules` builds probe suites (T-0008,
`eval/suites/probes-numeric`). One sweep per dependent task, each a spec suite under
`eval/suites/probes-numeric` (or `probes`) filled with `rustsheet-oracle fill` (excel-probe skill):
- T-0901 PV/FV/PMT/NPER/IPMT/PPMT/NPV on integer-`nper` and rate grids.
- T-0916 IRR/RATE/XIRR/MIRR: cash-flow families around the cached points, plus guess variations.
- T-0915 SIN/COS/TAN/SEC/CSC/COT at k·pi ± small offsets, for k up to 1e6.
- T-0908 BESSELJ/BESSELY for x in [8, 100] by orders 0..10, plus the order cap.
- T-0917 ASINH/ACOSH/ATANH/ACOTH/COTH/CSCH and the SINH/TANH switchover thresholds.
- T-0925 NORM.S.INV at p = NORM.S.DIST(k), tails down to 1e-300.

## Acceptance criteria
- [ ] Each listed family has an oracle-filled probe suite committed (no pending cells), and the dependent task file links it.
- [ ] New quirks visible in the sweeps are recorded in docs/excel-semantics/ (numbers.md, financial.md, engineering.md).
- [ ] Size: six families will not fit one M PR. Split per family at pickup with `make new-task`
  and repoint each dependent task to its own sweep, so each M9 task becomes ready as soon as its
  sweep lands.
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and
  no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, set this task `status: blocked` with the reason. The
  dependents then wait visibly instead of being fitted blind.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing any formulation (the dependent tasks do that).
