---
id: T-1100
title: "Tiny number text typed into a cell and in conditional-format rule values"
milestone: M1
lane: engine
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-1107]
---

## Goal
Typing tiny number text into a cell (`1e-310`, `1e-309`, `0e309`) and tiny numbers in
conditional-format rule values behave as Excel does, with evidence.

## Context
T-0154 put the small-magnitude rule (numbers.md "Number text of 1E+308 or more is not a
number": nonzero below 1E-309 is not a number, 1E-309 up to DBL_MIN reads as 0, zero mantissas
need exponent less fraction digits within ±308) in `parse_excel_number`
(crates/rustsheet-engine/src/number_format.rs), with evidence for coercion, VALUE, literals,
criteria, AVERAGE text, NUMBERVALUE and format conditions (probes/text-number-tiny*,
literal-number-tiny). Two paths have no evidence: cell entry, `set_user_input` via
`parse_formatted_number` (formatter/format.rs), which now stores typed `1e-310` as text; an
xlsx probe cannot show what typing does, so this needs a typed-input oracle path (Graph range
PATCH) or desktop Excel. And conditional formats (`parse_finite_number` in
conditional_formatting.rs) still read `1e-400` as 0, as does `value_needs_quoting` (utils.rs),
which now quotes the text "1e-310" though no reader takes it for a number (TODO(T-1100) in
crates/rustsheet-engine/src/test/test_nan_inf.rs).

## Acceptance criteria
- [ ] Evidence for what Excel stores when `1e-310`, `1e-309`, `2e-308` and `0e309` are typed into a cell, and for a conditional-format rule `Cell value > 1e-310`; numbers.md records it.
- [ ] `value_needs_quoting` agrees with `parse_excel_number` on tiny text, or a comment says why not.
- [ ] The engine follows it, reusing the T-0154 rule (no copy).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Sonnet run, no code written. Beyond sonnet: the first acceptance criterion needs a typed-input
oracle path, and none exists. `rustsheet-oracle status` shows excel-graph OK (app-only) and
excel-desktop n/a; `rg` over tools/ and docs/architecture/oracle.md finds no Graph range-PATCH
(typed value) support, and an xlsx probe cannot show what typing does. Next: add a typed-input path
to the oracle client (PATCH range values with the raw text, read back value/valueTypes), probe
`1e-310`, `1e-309`, `2e-308`, `0e309` and a CF rule `Cell value > 1e-310`, record in numbers.md,
then make `parse_formatted_number` / `parse_finite_number` / `value_needs_quoting` reuse
`parse_excel_number`.

## Blocked because
Opus run, 2026-10-07. The handoff is right: both halves of the evidence are out of reach today.
- Typed input: every oracle backend is xlsx in, xlsx out (`recalc`, tools/src/rustsheet_tools/oracle/backends.py),
  and stored values are not typed values. Adding a typed-input path is new oracle architecture, so it is
  filed as its own task, T-1107 (spec first), rather than done inside this S task.
- Conditional-format rule `Cell value > 1e-310`: no backend evaluates conditional formats (excel-graph cannot,
  excel-desktop is n/a), the same blocker as T-1051.
Unblock when T-1107 lands (cell entry), and for the CF half when excel-desktop or an Excel-authored workbook
with CF results is available. The `value_needs_quoting` criterion can be done earlier only if a reader that takes
"1e-310" as a number is shown; today none does (TODO(T-1100) in test_nan_inf.rs), so it waits with the rest.
