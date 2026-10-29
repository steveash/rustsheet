---
id: T-3107
title: "Probe T of long text through a range, name, INDIRECT, OFFSET, INDEX, IF, CHOOSE, LET and a sheet-qualified or legacy reference"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides what `T` gives for text over 255 UTF-16 units read through each reference shape, and ours matches.

## Context
T-2627 made `T(D1)` of a cell holding 256 units "" (probed) but left these shapes at `#VALUE!`, unprobed: `T(D1:D2)`, a defined name, `INDIRECT`, `OFFSET`, `INDEX`, `IF`, `CHOOSE`, `LET` and a spill reference. It is also implemented as "", unprobed, for a sheet-qualified reference, a legacy formula, `T((D1))`, `T(+D1)` and a single-cell structured reference. Probe each with the long text built by REPT in a cell (excel-probe skill; model on probes/text-255-units-from-cell), then align `t_function.rs` (`from_cell`) and update text.md "REPT, CODE, UNICODE and T refuse a text argument over 255 UTF-16 units".

## Acceptance criteria
- [x] A probe case with oracle values for each shape; `t_function.rs` matches or failures are owned; text.md lists each shape as verified.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe text-255-units-reference-shapes (excel-graph): T of a cell holding 256 units is "" through every reference shape; only `T(+D1)` is `#VALUE!`. `t_function.rs` now uses `reads_cell_text` (ISREF logic plus `@` and spill references, minus unary plus); 22/22 cells match; text.md updated.
