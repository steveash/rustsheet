---
id: T-3141
title: "CUMPRINC where v^nper underflows: the last bits of the balance-drop principal (T-3067's seven regressed near-1E15 cells and the underflow-ends misses)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3067]
---

## Goal
Type 0 CUMPRINC (and the CUMIPMT built on it) is Excel's to the bit where `v^nper` underflows, or the forms tried are
recorded in financial.md.

## Context
T-3067 run 5 found Excel's last-m-period principal does not depend on nper once `v^nper` underflows
(probes/fin-cumprinc-underflow-ends: nper 1E6, 1E10, 1E12 and 1E15 give the same doubles) and took the balance drop
`-(pv·(v^(nper-end)·(1-v^m)))` on Excel's EXP, Kahan's expm1 and `growth_log` there (`underflowed_principal` in
functions/financial/cumulative.rs; financial.md "T-3067 notes"). It is exact on 70 of the 108 underflowed principals
probed and within an ulp of the rest. Decision 0160 let it move these 7 cells off the bit; this task owns them:
- probes/fin-cumprinc-near-1e15 A7 (+1 ulp), B1, B4, B9, C10 (-1 ulp each), C6 (+1 ulp);
- probes/fin-cumprinc-near-1e15 E7 `CUMIPMT(0.01,1E15,100000,1E15-6,1E15,0)` (16 ulps, visibly off: A7's ulp through
  the payments less the principal).
- probes/fin-cumprinc-near-1e15 E2, E3, E4 `CUMIPMT(0.01,1E15,100000,1E15-k,1E15,0)` for k = 1, 2, 3 (Excel
  -29.60494069208835, -59.01479276444434, -98.03444828162765; ours -29.604940692088576, -59.01479276444388,
  -98.0344482816281: visibly off, the same cancellation of CUMPRINC's 1-ulp misses at A2, A3, A4; no open task named them,
  gap triage 2026-10-29).
Start with E7, the one visibly-off cell. Also open: the underflow-ends misses in A-D, G, H (each within an ulp), fin-cumipmt-overflow-and-forms A10 and the
type 1 ranges (column J, fin-cumipmt-period-bound A18), which still take the closed form. The implied L per cell is not
one biased log (the windows for m = 2 and m = 8 at 0.01 do not overlap), so the error is in the operation order; try
x87-register intermediates (`Register`) and the twice-rounded operators of numbers.md before more double orderings.
Decision 0056 applies (two attempts, then time-box).

## Acceptance criteria
- [ ] The 7 cells above pass (`make eval-baseline`) with no new known failures, or every form tried is in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges where `v^nper` does not underflow (T-3067, T-1374).
