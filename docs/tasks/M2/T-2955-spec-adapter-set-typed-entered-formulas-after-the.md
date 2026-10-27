---
id: T-2955
title: "Spec adapter: set typed (entered) formulas after the tables, so a typed structured reference resolves (probes/t2786 E5, E8, E9, E13:E15)"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-2801]
campaign: semantics
---

## Goal
A spec cell typed as `{"entered": "=..."}` is entered after the sheet's tables exist, as the formulas pass already
does for stored formulas, so a typed structured reference resolves instead of giving `#ERROR!`.

## Context
Found by the gap triager (2026-10-27 eval on 0f8dede8f). `spec_input` (crates/rustsheet-eval/src/engine.rs) maps
`CellInput::Entered` to `Input::Parse`, but the two-pass loop in `RustSheetEngine::evaluate_spec` only treats
`Input::StoredFormula`/`Input::LegacyFormula` as formulas (`is_formula`). A typed `=...` therefore goes in with the
values, *before* `add_table_with_totals` runs, so a typed `T[...]` cannot resolve and the cell is `#ERROR!`.
Only probes/t2786-whitespace-structured-refs-runs has typed formulas next to a table today (checked over every case),
and 14 of its 28 known failures are this or follow from it (it is 26 of the 28 visibly-off M0-M2 probe cells):

| cell | typed | Excel | ours |
|---|---|---|---|
| S!E5 | `=SUM(T[[Col]:[Other]])` | 77 | `#ERROR!` |
| S!E8 | `=SUM(T[[#Data], [Col]])` | 7 | `#ERROR!` |
| S!E9 | `=SUM( T[Col] )` | 7 | `#ERROR!` |
| S!E13 | `=T[#Headers]` | `#SPILL!` | `#ERROR!` |
| S!E14 | `=SUM(T[[#Data],[Col]:[Other]])` | 77 | `#ERROR!` |
| S!E15 | `=SUM(T[[#Data]  ,  [Col]])` | 7 | `#ERROR!` |

The triager checked a local patch that also counts `Input::Parse(t)` with `t.starts_with('=')` as a formula: E5, E8,
E9, E13, E14 and E15 then pass. These still fail after it, owned elsewhere:
- E4 (`=SUM(T[ [Col] ])`), E7 (`=SUM(T[ [#Data] , [Col] ])`) and E12 (`=SUM(T[ [Col]:[Other] ])`) are `#ERROR!`
  because the parser refuses whitespace inside structured-reference brackets. That is T-2801's work (its F4:F15 text
  needs the same parse); tell T-2801 in the PR, since its branch `agent/engine/T-2801-structref-whitespace` is open.
- C2:C4 are Excel's calculated-column fill (probe T-2957, engine T-2958).
- H1/H11 (typed NOW/TODAY scored) are T-2956.

**F8 is unmasked.** `=FORMULATEXT(E8)` passes today only because E8 never parsed and kept its typed text
`=SUM(T[[#Data], [Col]])`. Once E8 parses, ours prints `=SUM(T[[#Data],[Col]])` without the space Excel keeps after
the comma (syntax.md "Whitespace beside a missing argument is kept; a structured reference..."), which T-2801's
printing fixes. Either land after T-2801, or record the one unmasked line with a decision record, as 0034, 0037 and
0084 did. Never use `--allow-regressions` without one.

## Visible-parity stage
Target: S!E5, S!E8, S!E9, S!E13, S!E14 and S!E15 of probes/t2786-whitespace-structured-refs-runs have Excel's type and
value. All six cells are wrong-type (`#ERROR!` against a number or `#SPILL!`), so the stage is `semantics`.

## Superseded in flight (2026-10-27)
An opus run claimed this task and made the adapter change (a typed `=...` joins the formulas pass), with a
unit test: `make eval-check` then showed the 6 cells fixed and S!F8 regressed, as predicted. Meanwhile both open
T-2801 branches (`agent/engine/T-2801-structref-whitespace` and `agent/engine/T-2801-structured-ref-whitespace`,
2026-10-27 22:04 and 22:13 UTC) carry the same adapter change together with the printing that keeps F8 passing
(the second drops all 6 lines below from eval/baseline/probes.txt), so this task waits on T-2801 instead of landing
a duplicate with a decision record. When a T-2801 PR merges: if the 6 lines below are gone from
eval/baseline/probes.txt, set this task `done` with an Outcome naming that PR; otherwise do what is left, starting
from the adapter code T-2801 merged.
A unit test worth keeping if T-2801 has none: a spec with table T over A1:A3 (`Col`, 3, typed `(4)`) and typed
`=SUM(T[Col])` gives -1, typed `=A3*2` gives -8.

## Acceptance criteria
- [ ] Typed (`entered`) formula cells are entered in the formulas pass, after tables, pivots and
      structured-reference names. A typed non-formula (`{"entered":"(1)"}`) stays in the values pass. A
      rustsheet-eval unit test covers a typed `=SUM(T[Col])` next to a table.
- [ ] These 6 known-failure lines disappear from eval/baseline/probes.txt with no new ones, except S!F8, which goes
      only as described above (after T-2801, or with a decision record):
      `probes/t2786-whitespace-structured-refs-runs.case.json` S!E5, S!E8, S!E9, S!E13, S!E14, S!E15.
- [ ] `make eval` shows no other suite moves (no other case types a formula into a sheet with a table).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whitespace inside structured references, in values or text (T-2801).
- Calculated-column fill on typed entry (T-2957, T-2958).
- Skipping typed volatile formulas (T-2956).
