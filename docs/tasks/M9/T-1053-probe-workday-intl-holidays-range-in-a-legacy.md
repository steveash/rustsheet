---
id: T-1053
title: "Probe WORKDAY.INTL holidays range in a legacy formula (xlsx-authored, expect whole range)"
milestone: M9
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior is recorded for a legacy `=WORKDAY.INTL(C2,6,1,D2:D5)` (holidays range, expected whole range, 42438 for the poi/59106 layout).

## Context
Follow-up of T-0983 (review). WORKDAY.INTL's `holidays` is a whole-range argument by analogy with NETWORKDAYS.INTL and the Excel-cached WORKDAY case (poi/59106.xlsx Test!B2); no Excel evidence covers WORKDAY.INTL itself. Typed formulas never get implicit intersection, so the probe needs an xlsx-authored legacy formula (or an Excel-cached workbook).

## Acceptance criteria
- [x] Evidence for WORKDAY.INTL is recorded in docs/excel-semantics/dates.md ("WORKDAY holidays take the whole range"), which names WORKDAY.INTL.
- [x] The engine matches, or the gap is baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Escalated by a sonnet run without implementing (design question). `tools/src/rustsheet_tools/spec2xlsx.py` writes every
formula with `write_dynamic_array_formula`, and decision 0007 says legacy behavior is tested only through Excel-authored
workbooks, so a typed probe cannot produce a legacy `=WORKDAY.INTL(C2,6,1,D2:D5)`. Options to weigh: (a) a spec marker
for legacy (plain `write_formula`) cells in spec2xlsx plus a decision note on 0007; (b) hand-author a small xlsx and
fill it through the oracle's workbook path; (c) record only a dynamic-array probe (the holidays argument is a
range-typed parameter, so it is read whole there too) and say the legacy case stays unprobed. Then check the engine
(`args_signature_with_holidays`, static_analysis.rs) and update dates.md "WORKDAY holidays take the whole range".

## Outcome
Took handoff option (b): a hand-built fixture, `eval/suites/fixtures/cases/workday-intl-legacy-holidays.xlsx`,
with legacy `<f>` formulas (XlsxWriter `write_formula`, built per the fixtures README), filled by the
Excel oracle (excel-graph). WORKDAY.INTL, WORKDAY and NETWORKDAYS.INTL all read the holidays range whole
(42438 / 10), including from a row outside the range. A legacy `=D2:D5*1` control intersects to D3,
which shows the oracle kept the formulas legacy. The engine already matched (6/6 cells). Recorded in
dates.md "WORKDAY holidays take the whole range". Eval delta: fixtures +6 passing cells (new coverage),
0 regressions. No decision change was needed, since decision 0007 already routes legacy behavior
through hand-authored workbooks.
