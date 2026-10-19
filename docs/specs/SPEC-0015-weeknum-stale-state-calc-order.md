# SPEC-0015 — WEEKNUM's stale return_type state and Excel's fresh-load calc order

- **Status:** accepted
- **Milestone / lane:** M6 / engine
- **Tasks:** T-1484 (this spec and its probes), T-2068 (state and order), T-2069 (moved sheets),
  T-2070 (open probe questions)

## Problem
`WEEKNUM(serial, return_type)` with a return_type of 22 or more, or negative, reads a state that
earlier WEEKNUM and ISOWEEKNUM calls of the same recalc left behind (docs/excel-semantics/dates.md,
"WEEKNUM's stale return_type state"). `fn_weeknum` always gives `#NUM!`, which is Excel's value only
where the state is empty. With the T-1484 probes, 45 probes cells are known failures (all M6): 5 in
probes/weeknum-stale-return-type, 19 in -setters, 11 in -order-column-40, 1 each in the other
order, sheet, dependency and reset cases.

Decision 0038 rejected modelling this because the value depends on Excel's calc order, which was
unknown. Decision 0052 reopened it as a known gap. The T-1484 probes now pin down that order for
a freshly loaded workbook, which is the case the harness scores, and give a complete table of which
calls set, empty and read the state.

## Goals / non-goals
Goals:
- WEEKNUM with return_type 22+ or negative gives Excel's value on a freshly loaded workbook.
- No cost for workbooks without such a call: no change to the order or speed of their recalc.
- The result stays a deterministic function of the model (the cell contents and the load order of
  the sheets), restarts included.

Non-goals:
- Reproducing Excel's live-session history. In a session, formulas Excel recalculates alone, and
  formulas entered after loading, see an empty state or stop sharing it with the loaded ones
  (research note below). None of this is in the file, the harness never measures it, and RustSheet
  recalculates everything on each `evaluate()`.
- Importing `calcChain.xml`: Excel ignores it on a full-calc load (research note).
- Changing the order of anything else. The order is not observable except through this state.

## Excel behavior to match
From docs/excel-semantics/dates.md "WEEKNUM's stale return_type state" and recalc.md "The order
Excel calculates a freshly loaded workbook in" (excel-graph, 2026-10-19):
1. One state per recalc: `None`, `Sunday` or `Monday`. It starts at `None`.
2. A scalar WEEKNUM whose serial and return_type convert without error, and whose serial is valid
   (serial 0 included), updates it: types 1 (or omitted) and 17 set `Sunday`; 2, 11 and 21 set
   `Monday`; 12-16 and the always-`#NUM!` types 0, 3-10 and 18-20 set `None`. ISOWEEKNUM of a valid
   serial sets `Monday`.
3. A WEEKNUM that lifts over an array (either argument) sets `None`, whatever the types.
4. A WEEKNUM with return_type 22+ or negative reads the state: system-1 numbering from the state's
   weekday (the formula return_type 1 or 2 uses), or `#NUM!` when `None`. It does not change the
   state.
5. An error in the serial or a `#VALUE!` return_type (boolean, non-numeric text) leaves the state as
   it is; so do WEEKDAY and calls that IF does not run.
6. Order: formula cells run in the reverse of load order: sheets from the last loaded to the first,
   and within a sheet rows bottom to top, each row right to left. A precedent that has not been
   calculated runs just before the cell that reads it. A sheet keeps its load position after a move.

Evidence: the probes/weeknum-stale-* cases. The research note
docs/research/t1484-weeknum-calc-order-2026-10-19.md covers calcChain.xml and session edits.

Open (T-2070): a reader that lifts over an array, where a spilling formula sits in the order,
WEEKNUM inside a defined name, LET or LAMBDA, and conditional-formatting formulas. Until T-2070
answers them, T-2068 handles them as listed under Design.

## Design

### State
`Evaluation` (crates/rustsheet-engine/src/evaluation.rs) gets
`weeknum_start: Option<WeeknumStart>` (`enum WeeknumStart { Sunday, Monday }`, defined next to
`fn_weeknum`) and `weeknum_read: bool`. `run_pass` resets both with the other per-pass fields, so an
abandoned pass leaves nothing behind. Outside a pass (conditional formatting, formula helpers) the
state starts at `None` and is reset after each top-level evaluation, so a conditional format never
sees a value-cell's state (an assumption until T-2070).

`weeknum_of` becomes `&mut self` and applies rules 2, 4 and 5 after it has converted both
arguments: the match on the return_type that picks the start weekday also picks the state update.
Readers (22+, negative) look up `weeknum_start` and set `weeknum_read = true`. `fn_isoweeknum` sets
`Monday` after a successful `iso_week`. For rule 3, `fn_weeknum` sets `None` after a `lift_call`
that returned an array; a scalar call is not lifted. Each kernel call inside a lifted call still
updates the state, and the final reset gives rule 3's result. A reader that lifts reads the state
as it is at that element and is followed by the reset (open question, T-2070).

### Order, only when it matters
Reversing the driver's order for every workbook would make a long chain of formulas that each read
the one above (running totals) recurse to the top of the chain from its last cell. That costs depth
and unwinding on ordinary workbooks, for a function almost no workbook uses this way. So:
- The default pass keeps today's order (anchors, then forward storage order).
- When a pass completes and `weeknum_read` is set, `evaluate()` runs one more pass in **Excel order**
  and keeps that one. The extra pass happens only in workbooks where a reader ran. A new field,
  `Evaluation::excel_order: bool`, records that the model needs it, so later evaluations start in
  Excel order with no wasted pass. The field is cleared when a pass in Excel order completes with
  no reader.
- Excel order: dynamic anchors first, in `anchor_order` as today, then
  `all_positions()` reversed (sheet, row and column descending). A precedent is still evaluated on
  demand by `evaluate_cell`, which matches rule 6's "just before". Restarts work as they do now:
  each restarted pass resets the state and repeats the same order, so the result stays a function
  of the model.
- Where Excel puts a spilling formula is unknown (T-2070). Keeping anchors first is the existing
  behavior, so a reader inside a spilling formula is the one documented gap.

T-2069 replaces "sheet index" in the reversed order with the sheet's load rank. This is a
`load_rank: u32` per worksheet, set from the sheet's position when the workbook is read, and set to
the next free rank for sheets added later. It keeps probes/weeknum-stale-sheet-moved passing. Until
then, a moved sheet is ranked by its current index.

### Depth limit
Unwinding at `max_depth` replays a cell's evaluation. If a replay re-runs a WEEKNUM after other
cells have changed the state, it can give a different value from Excel's. This happens only past
the resource limit (directive 5). T-2068 adds a unit test of a reader in an unwound chain. If the
value differs, the task adds a `DEVIATION (decision 0014)` entry to dates.md.

### Public API
None. The state is internal to `Evaluation`; `evaluate()` keeps its signature.

## Alternatives considered
- *Always run in Excel order.* This is the simplest change, but it reorders every workbook and
  deepens recursion on running totals for one rare function. Rejected for the reason above.
- *Model the state in the current forward order* (decision 0038 option a). Its values are wrong
  for most layouts. Rejected.
- *Order by `calcChain.xml`.* Excel ignores the chain on a full-calc load (research note). Rejected.
- *Model live-session segments* (formulas entered after load sharing state only with each other).
  The file does not record which formulas were entered after load, so this would invent API
  semantics. Rejected; a non-goal.

## Test plan
- Eval: every probes/weeknum-stale-* cell passes (T-2068: all but -sheet-moved Second!A1, which
  T-2069 closes). The probes known failures drop by the count above. No other suite changes,
  because the extra pass runs only where a reader ran. `make eval` proves this: the
  ironcalc-calc and corpus-bench reports must be identical.
- Unit tests in crates/rustsheet-engine/src/test/: the setter/empty table (rules 2-5) on one sheet
  per rule. Reversed order across sheets. A reader after a restart caused by a spill. A workbook
  without readers runs exactly one pass (`restarts_in_last_evaluation` and a pass counter).
- Benchmark: the recalc benchmark (`make bench` or the T-0430 corpus timing) must not move for
  workbooks without WEEKNUM readers.

## Rollout
1. T-2068 (engine, opus, M): state, rules 1-5, Excel order on demand. Replaces the
   `TODO(T-2068)` in `fn_weeknum` and updates dates.md "Ours".
2. T-2069 (engine, sonnet, S, after T-2068): sheet load rank.
3. T-2070 (research, sonnet, S, independent): probes for the open questions. Rules it finds go to
   dates.md, and any change to this design becomes a follow-up task.
