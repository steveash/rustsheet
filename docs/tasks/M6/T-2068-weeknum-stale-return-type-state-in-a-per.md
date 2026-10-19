---
id: T-2068
title: "WEEKNUM stale return_type state in a per-recalc state and Excel's fresh-load order (SPEC-0015)"
milestone: M6
lane: engine
status: todo
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0015-weeknum-stale-state-calc-order.md
depends_on: []
---

## Goal
WEEKNUM with a return_type of 22 or more, or negative, gives Excel's value on a freshly loaded
workbook. It reads the per-recalc start-weekday state that earlier WEEKNUM and ISOWEEKNUM calls
leave, in Excel's fresh-load order.

## Context
SPEC-0015 has the rules and the design. docs/excel-semantics/dates.md "WEEKNUM's stale
return_type state" and recalc.md "The order Excel calculates a freshly loaded workbook in" have the
evidence. Every probes/weeknum-stale-* known failure is this task's except
probes/weeknum-stale-sheet-moved Second!A1, which is T-2069's. Code: `fn_weeknum`/`weeknum_of`
(functions/date_and_time/weeknum.rs, `TODO(T-2068)`), `fn_isoweeknum` (date_and_time.rs),
`Evaluation`, `Model::evaluate` and `run_pass` (evaluation.rs). The default pass order does not
change. A second pass in Excel order runs only when a reader ran (SPEC-0015 "Order, only when it
matters").

## Acceptance criteria
- [ ] The state follows SPEC-0015 rules 1-5: Sunday/Monday/empty, setters, emptying calls (array-lifted
      calls included), readers, and calls with no effect. Unit tests cover each rule.
- [ ] The Excel-order pass (anchors, then storage order reversed) runs only after a pass in which a
      reader ran. A workbook without readers runs the same single pass as before (tested).
- [ ] Every probes/weeknum-stale-* cell passes except -sheet-moved Second!A1. `make eval-baseline`
      records the fixes, and no other suite changes.
- [ ] The depth-limit replay case from SPEC-0015 "Depth limit" is tested, with a DEVIATION entry if
      its value differs from Excel's.
- [ ] dates.md "Ours" and the `fn_weeknum` comment describe the new behavior.
- [ ] `make check` green; reviewers approve (performance reviewer included).

## Out of scope
- Moved sheets' load rank (T-2069), and the open questions in T-2070.
- Live-session history (SPEC-0015 non-goals).
