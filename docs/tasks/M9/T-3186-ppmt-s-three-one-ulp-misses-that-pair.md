---
id: T-3186
title: "PPMT's three one-ulp misses that pair with PMT's own (ppmt-log-rate-sweep A371, A742; ppmt-first-period-sweep A320)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2243]
defer: phase-1b
---

## Goal
The three PPMT cells that T-2243's discounted split leaves one ulp off are bit-exact, with no other PMT or PPMT cell regressing.

## Context
- T-2243 made PPMT `-(A*Q(x)*d)` on PMT's own `A`, `1-Q` and `d` (financial.md "IPMT and PPMT split Excel's discounted
  payment"). That is bit-exact on 2552 of 2556 constant-argument PPMT cells. Of the misses, three are one ulp:
  - probes-numeric/ppmt-log-rate-sweep A371 `PPMT(0.21,1,160,10000)` (Excel -1.192779164112644E-10). The old form
    matched this cell, so it is the one cell T-2243 lost.
  - probes-numeric/ppmt-log-rate-sweep A742 `PPMT(1,1,44,10000)` (Excel -5.684341886081122E-10).
  - probes-numeric/ppmt-first-period-sweep A320 `PPMT(0.1,1,320,10000)` (Excel -5.679900781488781E-11).
- On each of those rows the PMT cell (B371, B742, B320) is also an ulp off with T-3014's PMT. So start from PMT's last
  bits (its `L`, `1-Q` or `A` there), not from the split.
- The fourth miss, `PPMT(7,10,350,58827)`, belongs to T-2666.
- Code: `discounted_balance`, `discounted_payment` and `discounted_split` in
  crates/rustsheet-engine/src/functions/financial/annuity.rs.

## Acceptance criteria
- [ ] The three cells and their paired PMT cells are bit-exact, with no regression across the PMT/PPMT sweeps or the baselines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2666's subnormal discount; CUMIPMT/CUMPRINC (T-3179).
