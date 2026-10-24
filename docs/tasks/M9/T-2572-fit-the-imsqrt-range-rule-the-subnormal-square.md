---
id: T-2572
title: "Fit the IMSQRT range rule, the subnormal-square flush in the IM log modulus and IMABS 1-ulp differences"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
IMSQRT's out-of-range answers, the IMLN/IMLOG10/IMLOG2 modulus for parts whose square is subnormal, and IMABS on
1E±150..155 pairs match Excel.

## Context
T-2117 switched IMLN, IMLOG10 and IMLOG2 to the `hypot` `modulus` (IMSQRT stays on `sqrt(x*x+y*y)`) and added the `#NUM!` rule for the three
logs. Still failing (docs/excel-semantics/engineering.md, "Not yet fitted"): IMSQRT `3E200+4E200i`, `1E155+1E155i`,
`1E160+0i` are `#NUM!` while `1E154+1E154i`, `1E-154+1E-154i`, `1E-160+0i` are "0";
`IMLN("1E-154+1E-153i")` is ln(1E-153) (a subnormal x² looks flushed to 0); IMABS of 1E±150..155 pairs differs from
Excel's value by 1 ulp. Cases: probes/eng-imln-modulus A17, eng-imln-modulus-range A43-A70, A79, A80
(`rustsheet-eval show probes/eng-imln-modulus-range.case.json`). Needs more probes to separate the
candidate modulus algorithms (scaled, flushed squares, long double).

## Acceptance criteria
- [ ] The cells above pass, or each remaining one is documented with probe evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMPOWER (T-1950).
