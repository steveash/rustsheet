---
id: T-2383
title: "Re-probe CELL('type',Data!C$9): one gen-m5 cell reads the referenced sheet, against the own-sheet rule"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
Excel's answer for `CELL("type",Data!C$9)` in gen-m5/74b183ca6fc1d190 is explained: either the own-sheet rule has an exception the engine can implement, or the cell is an oracle artefact and is re-filled.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `RIGHTB(CELL("type",Data!C$9),3)` is "l" in Excel, where Data!C9 is the text "abc" and Calc!C9 is blank, so this one cell reads the
referenced sheet. Ours follows the verified own-sheet rule (information.md "CELL(\"row\"/\"col\"/\"address\"/\"type\", range)",
T-1433) and gives "b". All 25 bare `=CELL("type",Data!..)` cells of the generated suites whose two sheets disagree at that address
fit the own-sheet rule, as do the "contents" cells. Observation: information.md, "Conflicting cell (T-2383)" paragraph (added by this
triage).
Steps: re-fill the case alone (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` on a copy holding only this
case, or `recalc`) to see whether the value repeats; then probe (`excel-probe` skill) `CELL("type",Data!C$9)` bare and under RIGHTB,
LEFT and `&""`, with `$` forms (`Data!C9`, `Data!$C9`, `Data!$C$9`), in rows above and below 9, and with Calc!C9 blank and filled.
If a rule comes out, file the engine change (functions-logic, `cell_info` in functions/information/mod.rs) as its own task.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/74b183ca6fc1d190 Calc!A347 | `=RIGHTB(CELL("type",Data!C$9),3)` | "l" | "b" |

## Acceptance criteria
- [ ] information.md states the outcome (a rule with evidence, or the artefact with the re-fill's result); a fix task is filed if the engine must change.
- [ ] Artefact: the re-filled sidecar removes this known-failure line from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite. Rule: the filed fix task owns the line (its Context lists the cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `fn_cell` (the follow-up task's).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.
