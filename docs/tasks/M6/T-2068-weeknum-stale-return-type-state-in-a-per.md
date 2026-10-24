---
id: T-2068
title: "WEEKNUM stale return_type state in a per-recalc state and Excel's fresh-load order (SPEC-0017)"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0017-weeknum-stale-state-calc-order.md
depends_on: []
---

## Goal
WEEKNUM with a return_type of 22 or more, or negative, gives Excel's value on a freshly loaded
workbook. It reads the per-recalc start-weekday state that earlier WEEKNUM and ISOWEEKNUM calls
leave, in Excel's fresh-load order.

## Context
SPEC-0017 has the rules and the design. docs/excel-semantics/dates.md "WEEKNUM's stale
return_type state" and recalc.md "The order Excel calculates a freshly loaded workbook in" have the
evidence. Every probes/weeknum-stale-* known failure is this task's except
probes/weeknum-stale-sheet-moved Second!A1, which is T-2069's. Code: `fn_weeknum`/`weeknum_of`
(functions/date_and_time/weeknum.rs, `TODO(T-2068)`), `fn_isoweeknum` (date_and_time.rs),
`Evaluation`, `Model::evaluate` and `run_pass` (evaluation.rs). The default pass order does not
change. A second pass in Excel order runs only when a reader ran (SPEC-0017 "Order, only when it
matters").

gen-m6 (T-2180, excel-graph oracle, 2026-10-21): 4 more cells whose return_type reads the stale state (a negative or
computed type; ours `#NUM!`), first filed under T-1484 and moved here when that spec landed:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/121a7c17092038d9 Calc!A238 | `=WEEKNUM(22503,SUM(Data!A6:A8,Data!D2:D4))` | 33 | #NUM! |
| gen-m6/3e51213f35889d06 Calc!A74 | `=WEEKNUM(367,YEAR(Data!E7))` | 1 | #NUM! |
| gen-m6/c96ac1ada46179c4 Calc!A219 | `=WEEKNUM(16038,SUM((Data!A3-7),Data!B11:B12,Data!E6:E9))` | 48 | #NUM! |
| gen-m6/c96ac1ada46179c4 Calc!A295 | `=WEEKNUM(26714,-ROUND(100,2))` | 8 | #NUM! |

## Acceptance criteria
- [x] The 4 gen-m6 lines above leave eval/baseline/gen-m6.txt, or each remaining one is named in the Outcome with its cause.
- [x] The state follows SPEC-0017 rules 1-5: Sunday/Monday/empty, setters, emptying calls (array-lifted
      calls included), readers, and calls with no effect. Unit tests cover each rule.
- [x] The Excel-order pass (anchors, then storage order reversed) runs only after a pass in which a
      reader ran. A workbook without readers runs the same single pass as before (tested).
- [x] Every probes/weeknum-stale-* cell passes except -sheet-moved Second!A1. `make eval-baseline`
      records the fixes, and no other suite changes.
- [x] The Excel-order pass is a chain: a read of an uncalculated formula cell aborts the cell,
      which moves to just after that precedent (SPEC-0017 rule 6; probes/weeknum-stale-precedent-*).
- [x] Probes and unit tests also pass with `IRONCALC_MAX_DEPTH=1` (SPEC-0017 "Depth limit").
- [x] Aborted runs restore the state. Blocked cells are skipped without evaluation. Total moves are
      capped at 4 × formula cells + 2^24, with on-demand fallback and a DEVIATION entry (SPEC-0017 "Cost").
      A test compares the chain against a naive simulation on small layouts.
- [x] A bench of a 100k-row running total plus one reader, with before/after numbers and a 10k vs
      100k ratio in the Outcome.
- [x] dates.md "Ours" and the `fn_weeknum` comment describe the new behavior.
- [x] `make check` green; reviewers approve (performance reviewer included).

## Out of scope
- Moved sheets' load rank (T-2069), and the open questions in T-2070.
- Live-session history (SPEC-0017 non-goals).

## Outcome
WEEKNUM's stale `return_type` state (`Evaluation::weeknum_start`) follows SPEC-0017 rules 1-5 in
`weeknum_of`, `fn_weeknum` (a lifted call empties it) and `fn_isoweeknum`. A completed pass in
which a reader ran is followed by a pass in Excel's fresh-load order (`excel_order.rs`): the
dynamic anchors first, as in the default pass (so its restarts, and `RestartLog`'s termination
proof, are unchanged), then the other formula cells as a linked-list chain in reverse load order,
where a read of an uncalculated formula cell stops the root with no effect (state restored) and
moves it just after that precedent. Moved cells remember their precedent and are moved again
without running while it is uncalculated; moves are capped at `4 × formula cells + 2^24`, past
which the pass evaluates on demand (DEVIATION entry in dates.md). `RestartLog` moved to
`restart_log.rs` to make room (evaluation.rs drops to 984 lines; its entry in the protected `quality/oversized-files.json` is left for a steward ratchet run).

New Excel quirk, from the gen-m6 cells this change first regressed: a spilling formula empties
the state at its place in the order (8 readers, all `#NUM!` in Excel; dates.md). The chain
empties it when it reaches an anchor's position. T-2070 gained a probe item for it.

Eval delta: probes known failures 2043 -> 1993 (50 fixed: every probes/weeknum-stale-* cell but
-sheet-moved Second!A1, T-2069's); gen-m6 151 -> 147 (the 4 cells listed above); no other suite
changes. Same results with `DEFAULT_MAX_DEPTH = 1` over the whole eval, and the unit tests pass
with `IRONCALC_MAX_DEPTH=1`.

Unit tests: test/test_weeknum_stale_state.rs (setters, emptying, no-effect calls, order within a
sheet and across sheets, stopped runs, re-runs after a precedent, spilling formulas, a restart, the
pass count, a 300-row running total, a cycle) and excel_order.rs (the chain against a naive
simulation of the rule on 500 random acyclic layouts, the quadratic reverse chain, the budget).

Bench (release, one run each, `evaluate()` first/second call): running total `=A{n-1}+1` with a
reader in B1 and a Monday setter in B{n}: before 1.5 ms (10k) and 23 ms (100k), reader `#NUM!`;
after 34 ms (10k) and 75 ms (100k), reader 16. Both sizes exhaust the move budget, whose 2^24
floor (about 30 ms) dominates: 100k/10k = 2.2. Without a reader nothing changes (1.9 vs 1.7 ms
at 10k, 22 vs 27 ms at 100k, noise).
