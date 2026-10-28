---
id: T-2713
title: "TEXTJOIN walks a single range's stored cells through the shared walker, not its own dense loop (TEXTJOIN over A:J beside one cell in J1048576 takes 4 s)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-2710]
---

## Goal
TEXTJOIN over a one-sheet range costs the range's stored cells, as the other aggregates do after
T-2710, with the same text, delimiters, length limit and errors.

## Context
Gap triage 2026-10-25 (robustness, follow-up of T-2710). `=TEXTJOIN(",",TRUE,A:J)` on a sheet whose only
other cell is J1048576 takes 4.1 s (release, scratch timing), and `CONCAT(A:J)` 7.2 s. CONCAT walks
through `for_each_cell_or_error_in_areas` (crates/rustsheet-engine/src/functions/text/concatenate.rs
`fn_concat`), and TEXTJOIN's 3-D path walks through `for_each_cell_or_error_in_whole_range`. T-2710
makes both sparse. TEXTJOIN's one-sheet path keeps its own `for row ... for column ... evaluate_cell`
loop after `clip_to_used_area` (`fn_textjoin`, the `CalcResult::Range` arm). With ignore_empty FALSE
that loop pads one delimiter per skipped blank (`pad`, `skipped_columns`, `skipped_rows`).

Evidence the result must keep (docs/excel-semantics/text.md): probes/whole-column-empty-walkers F1, F8
(`TEXTJOIN(",",FALSE,D:D)` is #CALC!, `TEXTJOIN(",",FALSE,10:10)` has 16,383 delimiters) and
probes/text-result-limit-errors-refs (an error cell past the limit is dropped).

## Acceptance criteria
- [ ] `fn_textjoin`'s one-sheet range arm uses T-2710's sparse walker (or its blank count), with no dense
      loop of its own left. With ignore_empty FALSE, the blank positions inside the used area still add
      their delimiters, counted, not visited one by one.
- [ ] A unit test with a work counter (T-2710's) shows `TEXTJOIN(",",TRUE,A:J)` and `CONCAT(A:J)` beside
      one cell in J1048576 visit a few cells and give "1".
- [ ] All TEXTJOIN and CONCAT cells in eval pass as before (no baseline gains a line).
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN over unions (`#VALUE!`, SPEC-0004) and the 32,767-unit limit rules.
