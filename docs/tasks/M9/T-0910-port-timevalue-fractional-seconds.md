---
id: T-0910
title: "Port IronCalc TIMEVALUE fractional-seconds fix"
milestone: M9
lane: functions
status: todo
priority: P3
size: S
depends_on: []
---

## Goal
Upstream IronCalc accepts fractional seconds in TIMEVALUE (`Ported-From: ironcalc@cead0e28`,
follow-up test cleanup `13dbb103`). Port it, but only after confirming Excel's behavior.

## Acceptance criteria
- [ ] Excel evidence for `TIMEVALUE("10:11:12.5")` and neighbours (cached values in an
  Excel-authored workbook, or oracle via `excel-probe`); cite it in `docs/excel-semantics/`.
- [ ] Spec case(s) added under `eval/suites/`; implementation matches; `make check` green.
- [ ] Commit message carries `Ported-From: ironcalc@cead0e28`.
