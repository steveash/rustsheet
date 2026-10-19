---
id: T-1430
title: "Probe DATEDIF and DAYS360 error order when a date and the unit or method are both bad"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's error order is known for DATEDIF and DAYS360 when a date argument and the unit (DATEDIF) or method (DAYS360)
are both bad, and ours matches it.

## Context
T-1235 kept each function's pre-lift order without oracle evidence. Ours today: `DATEDIF(10,5,NA())` is `#NUM!`
(the dates' #NUM! beats the unit's error), `DATEDIF(1/0,5,NA())` is `#DIV/0!`, `DAYS360(-5,5,NA())` is `#N/A`
(the method's error beats the dates' #NUM!), `DAYS360(1/0,5,NA())` is `#DIV/0!`. The DATEDIF/DAYS360 orders differ
from each other, so at least one may be wrong.
Semantics: docs/excel-semantics/dates.md "DATEDIF and DAYS360 error order with a bad unit or method (inferred)".
Code: `fn_datedif` / `datedif_value` and `fn_days360` in crates/rustsheet-engine/src/functions/date_and_time.rs.
Use the `excel-probe` skill; add the four formulas (and their lifted variants, e.g. `DATEDIF({10,1},5,NA())`)
to a probe case.

Evidence from T-1785: `INDEX(DATEDIF(46000,45000,{"d";#N/A}),2)` is `#N/A` in Excel (probes/date-math-lift-driver-edges G43, a known failure: ours gives the reversed pair's `#NUM!`), so the unit's error may come before the reversed-pair check. The DATEDIF and DAYS360 kernels are in `fn_datedif`/`fn_days360` on the SPEC-0011 driver.

## Acceptance criteria
- [x] A probe case holds `DATEDIF(10,5,NA())`, `DATEDIF(1/0,5,NA())`, `DAYS360(-5,5,NA())`, `DAYS360(1/0,5,NA())`
  with oracle values; ours matches them (baseline updated).
- [x] dates.md's entry is VERIFIED (or records the difference) and the code comments cite the probe.
- [x] probes/date-math-lift-driver-edges Sheet1!G43 (`=INDEX(DATEDIF(46000,45000,{"d";#N/A}),2)`, Excel #N/A) passes; the baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New probe probes/datedif-days360-error-order (41 cells, excel-graph): both functions read every
argument's value first (start, end, then unit/method), and only then check the dates' range. DAYS360
already matched. DATEDIF checked a reversed pair before the unit's error; `datedif_value` now reads
the unit's error first, so `DATEDIF(10,5,NA())` is #N/A. dates.md's entry is VERIFIED.
Eval: probes known failures 1812 → 1811 (date-math-lift-driver-edges G43 fixed), a new case of 41
passing cells, 0 regressions.
