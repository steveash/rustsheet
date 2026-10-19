---
id: T-1892
title: "Probe IMPOWER/IMDIV edge cases left UNVERIFIED by T-1766: huge angle, underflow, blank first argument"
milestone: M9
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The IMPOWER/IMDIV behaviors T-1766 implemented without evidence are settled by Excel oracle values, and the code matches them.

## Context
docs/excel-semantics/engineering.md "T-1625 probes" lists three UNVERIFIED items: (1) IMPOWER with an angle `atan2(y,x)·n` of 2^27 or more falls back to libm sine/cosine (`fn_impower` in crates/rustsheet-engine/src/functions/engineering/complex.rs; SIN is `#NUM!` from 2^27, `EXCEL_TRIG_LIMIT`); (2) a blank first argument in IMDIV and IMPOWER reads as 0 (only IMSUB is evidenced, COMPLEXs.xlsx Sheet2 G7:G10); (3) IMDIV uses Smith's algorithm, which may differ from Excel in the last ulp. Also unprobed: IMPOWER of an underflowing radius (`IMPOWER("1E-200",2)`), now through `arithmetic::power`. Write a probe case per the `excel-probe` skill (e.g. `IMPOWER("i",134217728)`, `IMPOWER("1+i",170891318)`, `IMPOWER("1E-200",2)`, `IMPOWER("1E-200+1E-200i",-2)`, `IMDIV(blank,"1+i")`, `IMPOWER(blank,2)`, IMDIV with non-trivial divisors like `IMDIV("1.1+2.3i","3.7-0.9i")` across a sweep), get oracle values, then fix what differs.

## Acceptance criteria
- [ ] A probe case with oracle values covers the cases above; engineering.md entries flip from UNVERIFIED to the evidenced rule.
- [ ] Every cell of the new probe passes, or a follow-up task owns the rest.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
