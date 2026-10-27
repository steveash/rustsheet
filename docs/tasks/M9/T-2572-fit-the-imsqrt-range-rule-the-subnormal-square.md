---
id: T-2572
title: "Fit the IMSQRT range rule, the subnormal-square flush in the IM log modulus and IMABS 1-ulp differences"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: M
tier: opus
campaign: visible
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

## Visible-parity stage
Decision 0083/0114: this task is done when every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move to a follow-up task filed without the key and with `defer: phase-1b`.
Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: probes/eng-imln-modulus-range A79 `=IMLN("1E-154+1E-153i")`: Excel "-352.295519228089+1.47112767430373i", ours "-352.290544062662+1.47112767430373i"; A80 `=IMSQRT("1E-154+1E-153i")`: Excel "2.34467878292635E-77+2.12190513569646E-77i", ours "2.35051862586971E-77+2.12719012092489E-77i".
