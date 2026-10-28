---
id: T-2956
title: "Skip a typed (entered) NOW or TODAY formula as volatile, as a stored one is (probes/t2786 H1, H11)"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
A typed (`{"entered": "=NOW( )"}`) volatile formula in a spec case is skipped like a stored one, while a
FORMULATEXT of it, which does not depend on the clock, stays scored.

## Context
Found by the gap triager (2026-10-27 eval on 0f8dede8f). `CellInput::formula()` (crates/rustsheet-eval/src/case.rs)
returns `None` for `CellInput::Entered`. So the runner's `formulas` list (runner.rs, input of `spec_volatile_cells`)
and `formula_of` never see a typed formula, a typed `NOW`/`TODAY` is scored against the oracle's clock, and
`make eval-show` prints its formula as a bare `=`. probes/t2786-whitespace-structured-refs-runs is the only case with
a typed volatile formula (checked over every case):
- S!H1 `=NOW( )`: Excel 46321.249193055555, ours 44873.467691875 (the run's clock);
- S!H11 `=TODAY(  )`: Excel 46321, ours 44873.

Two of the 28 visibly-off M0-M2 probe cells. They are not evidence of anything: a stored NOW is skipped (T-0005,
"Spec workbooks have no METADATA sheet, so the clock is never pinned").

Watch the taint: `volatile_cells` (volatile.rs) taints every formula that refers to a volatile cell, so once H1 counts
as a formula, `=FORMULATEXT(H1)` in S!I1 and `=FORMULATEXT(H11)` in S!I11 would be skipped too. Those two cells are
Excel evidence that T-2944 needs (its acceptance names I1:I13: Excel prints `=NOW( )` and `=TODAY(  )`). FORMULATEXT
and ISFORMULA read a cell's formula, not its value, so a reference that is only their argument must not carry the
taint. Keep the rule that narrow: any other use of the reference, such as `FORMULATEXT(H1)&H1`, still taints.

## Visible-parity stage
Target: S!H1 and S!H11 of probes/t2786-whitespace-structured-refs-runs are skipped as volatile (their F lines leave
eval/baseline/probes.txt) and S!I1 and S!I11 stay scored. The stage is `semantics`: the cells are wrong because of a
harness rule, not a numeric kernel.

## Acceptance criteria
- [ ] `CellInput::formula()` returns the formula of a typed `=...` entry. `make eval-show
      CASE=probes/t2786-whitespace-structured-refs-runs.case.json` prints the typed formula, not `=`.
- [ ] Unit tests in volatile.rs: a typed `NOW()` is tainted, `FORMULATEXT(H1)` of it is not, and
      `FORMULATEXT(H1)&H1` is.
- [ ] These 2 known-failure lines disappear from eval/baseline/probes.txt with no new ones:
      `probes/t2786-whitespace-structured-refs-runs.case.json` S!H1 and S!H11. The case's skipped-volatile count is 2,
      and S!I1 and S!I11 are still checked (their lines stay until T-2944).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Entering typed formulas after the tables (T-2955).
- Printing the kept whitespace in I1:I13 (T-2944).
