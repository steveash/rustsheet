---
id: T-3656
title: "INDIRECT: probe and match reference text with spaces and reversed ranges"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDIRECT resolves, or refuses with Excel's error, reference text that holds spaces or a reversed range, in A1 and R1C1 style.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports INDIRECT failing on reference text with spaces or a reversed range. Known:
lookup.md INDIRECT sections (missing sheet, structured references, names, a1 coercion, R1C1 offsets), references.md
"INDIRECT does not take 3-D text", T-1138 (reversed sheet-qualified range corners). Open: T-3394 (structured
references, includes spacing), T-2347 (1x1 array argument). Not probed: leading and trailing spaces ("  A1 "), spaces
around the colon ("A1 : B2"), between sheet and !, inside a quoted sheet name, a space-separated pair ("A1 B2", the
intersection operator in a formula), tabs and NBSP; reversed ranges in every direction ("B2:A1", "A2:B1", "B1:A2"),
reversed whole columns and rows ("C:A", "3:1"), reversed R1C1 ("R2C2:R1C1"), and a reversed range over two sheets'
text. Check what the reversed range normalises to with ROWS/COLUMNS/CELL("address") and SUM. Code: `fn_indirect` in
functions/lookup_and_reference/mod.rs and the reference-text parser it calls.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the shapes above in A1 and R1C1 style.
- [x] The engine matches every probed cell; lookup.md states the rules.
- [x] Structured-reference spacing is left to T-3394.
- [x] Every quirk found is written up in docs/excel-semantics/lookup.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Structured-reference text (T-3394).

## Outcome
Probe `eval/suites/probes/cases/t3656-indirect-spaces-reversed` (excel-graph, 2026-11-01): 61 reference texts,
each read with SUM, ROWS, COLUMNS and (same-sheet only) CELL("address"). Two engine bugs found and fixed:
- Whitespace: INDIRECT refuses leading whitespace, tabs, NBSP, a space before `!` and a space-separated pair,
  and drops trailing spaces/LF and spaces next to `:` or after `!`. The engine's lexer skipped all of them and
  refused the colon spacing. New `indirect_reference_text` (expressions/utils) runs before both parsers.
- Reversed A1 ranges (`B2:A1`, `C:A`, `3:1`, `Sheet2!B2:A1`) kept their corners reversed (SUM 0, ROWS
  1.8e19); `indirect_of` now orders them with `Range::between`. R1C1 already ordered them.
Written up in lookup.md ("INDIRECT reference text: where spaces are allowed", "INDIRECT of a reversed range
is the range with its corners in order"). Web-only missing-sheet cells (resolved as external workbooks) were
left out of the case. Structured-reference spacing stays with T-3394.
