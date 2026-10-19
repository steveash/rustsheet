---
id: T-1892
title: "Probe IMPOWER/IMDIV edge cases left UNVERIFIED by T-1766: huge angle, underflow, blank first argument"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
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

## Outcome
Probes `eng-impower-imdiv-edges` (35 cells) and `eng-impower-angle-cut` (19 cells), excel-graph. Excel's IMPOWER angle cut is just under 2^27 (bracketed at 134217727), not SIN's 2^27, and gives `#NUM!` for a pure imaginary base; an underflowed radius gives "0" whatever the angle. Our libm fallback is now `#NUM!`. A blank first argument as 0 and Smith's IMDIV were confirmed bit for bit. A non-imaginary base past the cut gives values near 1.797E+308 in Excel, which we do not reproduce: follow-up T-1950.

## Handoff
Sonnet run: the work is done and `make check` is green on the branch; reviewers security, architecture and quality APPROVE. Excel-fidelity requested changes in three rounds (round 1: wrong 2^27 evidence and an unevidenced radius-0 short-circuit; round 2: modulus-power finding unrecorded; round 3: an uncommitted "n=1000 to 80000000" sweep claim, since deleted). Every item raised is fixed in the last commit; it has not been re-reviewed.
Next: (1) re-run `reviewer-excel-fidelity` on the branch head; (2) if it approves, set `status: done`, run `make check`, mark the PR ready and merge under the steward policy; (3) T-1950 owns the open Excel differences (modulus power near radius 1, non-imaginary base past the cut). The IMPOWER cut constant 134217727 is a bracket from probes (85445658 answers, 85445659 is `#NUM!`), not a derived value.
