---
id: T-2111
title: "Milestone function scope reports a cumulative ironcalc-calc row per function milestone (M5 exit criterion)"
milestone: M5
lane: harness
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`make eval` reports, for each function milestone in phase order (M2, M5, M6, M7, M8, M9, M10), how
the ironcalc-calc cells score whose functions all have a catalog milestone up to and including that
one. The M5 exit criterion "100% on ironcalc-calc for these categories" (docs/milestones.md) is then
a reported number with its failing cells listed, as M2's is today. M6 reuses it ("same as M5 for
these categories").

## Context
- Today crates/rustsheet-eval/src/function_scope.rs computes one row, with a fixed
  `MAX_MILESTONE = 2` (T-1472, T-1877, T-1970). The report section is "Milestone function scope
  (ironcalc-calc)"; on 2026-10-20 it shows 4245 scored, 4244 bit-exact and 1 failing
  (`LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`, the decision 0074 exception).
- Rule for row Mk: a cell is in scope when every function it calls (its own formula, or its anchor's,
  plus the defined names it calls, exactly as today) has catalog milestone ≤ k. Functions the catalog
  does not list count as in scope, and a formula with no function call counts in every row; both as
  today. Milestones compare numerically through `probe_milestones::parse_milestone` (M3/M4 have no
  catalog functions, so ≤ 5 means M0–M2 and M5).
- A 2026-10-20 planner script over eval/runs/latest/report.json estimates the M5 row's new failing
  cells at about 53. They are COTH, ACOTH, CSCH, ATANH, TANH, SINH, SECH, COT and CSC cells of
  `MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` (T-0917, T-0915), `ATAN2_POWER.xlsx` D14 and D26,
  `EXP_SIGN.xlsx` B44 (T-1944), `LOGICAL/IF_ARRAY.xlsx` C4 and E4, and `TEXT/T_VALUE_VALUETOTEXT.xlsx`
  I4 (T-0929). The script's regex did not expand defined names, so treat the count as approximate.
- Keep the failing lists readable: under each row, list only the failing cells that no earlier
  (lower) row already lists. Each table row still counts all its cells.

## Acceptance criteria
- [ ] report.md's "Milestone function scope (ironcalc-calc)" section has one table row per function
      milestone M2, M5, M6, M7, M8, M9 and M10, with the same columns as today plus the milestone. Each
      row lists its newly failing cells as described above. report.json carries the rows as a list,
      each with its milestone.
- [ ] The M2 row is unchanged from the 2026-10-20 run: 4245 scored, 4244 bit-exact, 1 failing (C11).
- [ ] Unit tests: an M5-only formula is in the M5 row and not in M2; a formula calling an M6 function
      is in neither; a function-free formula is in every row; the defined-name and anchor tests still pass.
- [ ] The milestone list comes from the catalog's milestones, or from one constant shared with
      `probe_milestones`. Do not add a second milestone parser (CLAUDE.md directive 3).
- [ ] The Outcome records the M5 row's numbers on the PR's tree and maps each failing family to its
      open task id, or says it has none.
- [ ] docs/architecture/eval-harness.md describes the section.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deciding whether M5's 100% exempts any cell. That is T-2116, the decision record.
- Following precedents transitively.
- Suites other than ironcalc-calc.

## Outcome
`function_scope` now reports one cumulative row per report row of the probe milestones (`REPORT_ROWS`, shared with `probe_milestones`: M2, M5 to M11, plus any other catalog milestone), in report.md ("Newly failing at Mk" lists each failing cell once) and report.json (`function_scope.rows`). M8, M10 and M11 have no catalog functions of their own, so their rows equal the one before. Docs: eval-harness.md "Milestone function scope". Unit tests cover the M5-only, M6-call and function-free formulas, array anchors and defined names.

`make eval` on this tree: M2 row unchanged (4245 scored, 4244 bit-exact, 1 failing, `IF_ARRAY.xlsx!Sheet1!C11`, decision 0074). M5 row: 60129 scored, 60075 bit-exact, 0 deviations, 54 failing (53 new beyond M2). M6: 63179 / 63125 / 54. M7: 64075 / 64020 / 55 (adds `POWER.xlsx!inside_a_lambda!C3`). M9: 72209 / 71162 / 1047.

M5's 53 new failures by family: TRIGONOMETRIC.xlsx COTH/CSCH/SECH, ACOT/ACOTH, ASINH/ACOSH/ATANH and SINH/COSH/TANH cells: T-0917; TRIGONOMETRIC.xlsx COT/CSC/SEC B47 and C48: T-0915; `EXP_SIGN.xlsx!EXP_SIGN!B44`: T-1944; `T_VALUE_VALUETOTEXT.xlsx!Sheet1!I4`: T-0929; `IF_ARRAY.xlsx` C4 and E4 and `ATAN2_POWER.xlsx` D14 and D26: no open task (T-2116 decides whether M5 exempts any cell).
