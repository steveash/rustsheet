---
id: T-1678
title: "Keep the picked-argument index of XLOOKUP, TAKE, DROP and TRIMRANGE next to Function::picks_reference"
milestone: M2
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The list of reference-picking calls (XLOOKUP, TAKE, DROP, TRIMRANGE) and the argument each one picks from are defined in one place. A fifth function added to the list cannot silently use the wrong argument.

## Context
Found by the 2026-10-16 steward spot-audit of PR #500 (T-1252). The opus architecture, quality and fidelity reviewers each flagged it as MINOR.
- `Function::picks_reference` (`crates/rustsheet-engine/src/functions/mod.rs`) holds the call list.
- `is_read_reference` (`expressions/parser/static_analysis/aggregate.rs`) still picks the argument inline: 2 for XLOOKUP, 0 for the others.
- Suggested shape: `Function::picked_reference_arg(&self) -> Option<usize>`, with `picks_reference()` being `.is_some()`. The aggregate static analysis uses the index; `bind_argument` (let.rs) and `evaluate_picked_reference_arg` (util.rs) keep the bool.

## Acceptance criteria
- [ ] One `Function` method gives both the call list and the picked argument index; `aggregate.rs` has no hard-coded index.
- [ ] Pure refactor: every eval suite shows 0 fixed and 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding new picking functions (CHOOSEROWS and others need probes first).

## Outcome
`Function::picked_reference_arg` (functions/util.rs) is the one list of reference-picking calls and the argument each picks from (XLOOKUP 2, TAKE/DROP/TRIMRANGE 0); `picks_reference` is `.is_some()` and `aggregate.rs` reads the index from it. The methods moved out of the oversized `functions/mod.rs`, which shrank 2182 -> 2172 lines (`quality/oversized-files.json` updated with the script). Pure refactor: every eval suite 0 fixed, 0 regressions.
