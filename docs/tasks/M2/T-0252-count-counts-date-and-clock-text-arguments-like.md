---
id: T-0252
title: "COUNT counts date and clock text arguments like SUM coerces them"
milestone: M2
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`COUNT` counts a direct text argument whenever Excel reads that text as a number, including clock and date text: `COUNT("10:00","x")` is 1 in Excel (the engine gives 0).

## Context
Found in T-0153. Evidence: probes/clock-text-oddities F37 (excel-graph, baselined in eval/baseline/probes.txt). `fn_count` (crates/rustsheet-engine/src/functions/statistical/count_and_average.rs) tests direct text with `parse_finite_number` only, while SUM coerces with `cast_number` (cast.rs), which also reads clock, date, currency and percent text. Before switching COUNT to `cast_number`, probe the other forms (`COUNT("1/2/2020")`, `COUNT("$1")`, `COUNT("1%")`, `COUNT("1,000")`, `COUNT(" 1")`, `COUNT("(1)")`, `COUNT("6 PM")`) with the excel-probe skill; keep the "inf"/1e308 rules in docs/excel-semantics/numbers.md. See docs/excel-semantics/dates.md "Time text" (direct-argument bullet).

## Acceptance criteria
- [ ] Probe case for the text forms above, oracle-filled.
- [ ] COUNT matches every probe cell; clock-text-oddities F37 leaves eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_count` coerces direct text with `cast_number` (finite only). probes/count-direct-text added (13 cells, oracle-filled, all pass); clock-text-oddities F37 fixed. Parenthesised text `(1)` = -1 is a separate gap, filed as T-0257 with probes/paren-negative-text. probes known failures 108 -> 111 counting the 4 new-coverage failures of that probe, -1 fixed.
