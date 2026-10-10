---
id: T-0285
title: "Probe CONCAT/TEXTJOIN cell errors past 32,767 through names, INDEX/IF/OFFSET/INDIRECT, @ and 3-D references"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what CONCAT and TEXTJOIN return when a cell error reaches them past the 32,767-character limit through a reference that is not a direct A1 cell or range, and the engine matching it.

## Context
T-0284 probed direct references (probes/text-result-limit-errors): a cell error read once the text, plus for TEXTJOIN the preceding delimiter, is over 32,767 is dropped and the result is `#CALC!`. Unprobed (docs/excel-semantics/text.md, "CONCAT and TEXTJOIN past 32,767 characters"): defined names, INDEX/IF/OFFSET/INDIRECT results, `@A1:A3`, and 3-D references. The engine differs by function: `fn_concat` reads them through `evaluate_aggregate_arg` as references (error dropped, `#CALC!`), while `fn_textjoin` only drops the error for a direct `Node::ReferenceKind` and otherwise returns it. Code: functions/text/concatenate.rs. Use the `excel-probe` skill; reuse the T-0284 layout (`A1` = "b", `A2` = `#N/A`, prefix `REPT("a",32767)`).

## Acceptance criteria
- [x] Probe cells with oracle values for CONCAT and TEXTJOIN through a defined name, INDEX, IF, OFFSET, INDIRECT, `@` and a 3-D reference; the text.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-result-limit-errors-refs` (excel-graph, 24 cells): every reference kind behaves like a direct A1 reference (range with "b" gives `#CALC!`; single `#N/A` cell gives `#N/A` for CONCAT, `#CALC!` for TEXTJOIN). `@A1:A2` from rows 9/10 is `#VALUE!` and says nothing about the limit. Engine: TEXTJOIN now reads arguments through `evaluate_aggregate_arg` and walks 3-D references; `:` range expressions are aggregate references and an INDEX operand of `:` stays a reference. Probe 18/24 -> 24/24 with no regressions (known failures unchanged, one new passing case). text.md updated.
