---
id: T-2936
title: "BETAINV(1E-15,1E+8,1) is 1-3.45031E-7 in Excel, not p^(1/alpha)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`BETAINV(1E-15,1E+8,1)` has Excel's value to 15 significant digits, so gen-m9 5ded7594e597e31e Calc!A177
(`=IPMT(BETAINV(Data!C8,1E+8,COLUMN()),1.5,77,75622)`) matches Excel.

## Context
- Split from T-2587, which found that A177's miss is not IPMT's. With beta 1 the quantile is `p^(1/alpha)`:
  ours 0.9999996546122957 (`exp(ln(1E-15)/1E+8)`), and IPMT of it, about `-rate*pv`, is ours -75621.97388109102.
  Excel's IPMT -75621.97390807692 implies a BETAINV of about 1-3.45031E-7 (ours 1-3.45388E-7), a 3.6E-10
  relative difference, so Excel's BETAINV at shape 1E+8 is not the closed form.
- First step: probe `BETAINV(1E-15,1E+8,1)` and neighbours (alpha 1E+5 to 1E+9, p 1E-15 to 0.5) directly
  (excel-probe skill). Code: crates/rustsheet-engine/src/functions/statistical (beta quantile, `bisect_unit_interval`
  above shapes 1E+5, T-1543).
Cells:
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A177

## Acceptance criteria
- [ ] The probe is oracle-filled and statistics.md records Excel's BETAINV at huge alpha.
- [ ] Calc!A177 within half a unit of the 15th significant digit, no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETAINV's underflow and other tails (T-2808).
