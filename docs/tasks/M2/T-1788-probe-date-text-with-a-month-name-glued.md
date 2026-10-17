---
id: T-1788
title: "Probe date text with a month name glued to digits (July1, January0, 1JUN2021)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's reading of date text whose month name touches digits with no separator is known well enough
to implement it without guessing: an excel-probe case scores the glued forms in DATEVALUE, VALUE,
MONTH and number coercion.

## Context
corpus-bench shows three glued forms parse in Excel (docs/excel-semantics/dates.md, "Date text with a
month name glued to digits"): `"July1"` (corpus-bench/1726 `July-KPIs!J12`, J13), `"January0"`..
`"December0"` (corpus-bench/55392 `Monthly Expenses Summary!E6:P30`) and `"1JUN2021"`
(corpus-bench/CF_18814 `attendance-Dec!E1`). The engine's grammar (`formatter::date_text::parse_date`)
has no glued forms, and three examples do not say which digits are a day and which a year.
Probe at least: `"July1"`, `"July10"`, `"July31"`, `"July32"`, `"July45"`, `"July0"`, `"Jul1"`,
`"Jul2026"`, `"July2026"`, `"July1,2026"`, `"July12026"`, `"1July"`, `"1Jul2026"`, `"01JUN21"`,
`"1June2021"`, `"1jun2021"`, `"Jun1 2021"`, `"1JUN 2021"`, `"1 JUN2021"`, `"Sept1"`, `"Ju1"`, each in
DATEVALUE, VALUE, `MONTH()` and `0+`, with clock-independent cells (MONTH/DAY/YEAR) scored and the
current-year serials left unscored, as probes/date-text-yearless does. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probes case (e.g. `probes/date-text-glued-month`) with an excel-graph sidecar covers the forms above.
- [ ] docs/excel-semantics/dates.md "Date text with a month name glued to digits" states the rule from that evidence and cites its cells.
- [ ] T-1789 is updated with the rule (or split) if the evidence differs from its assumption.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The parser change (T-1789).
