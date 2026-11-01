---
id: T-3656
title: "INDIRECT: probe and match reference text with spaces and reversed ranges"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDIRECT resolves, or refuses with Excel's error, reference text that holds spaces or a reversed range, in A1 and R1C1 style.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, branch `claude/sharp-mendel-13x2km`). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

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
- [ ] A probe case (excel-graph) covers the shapes above in A1 and R1C1 style.
- [ ] The engine matches every probed cell; lookup.md states the rules.
- [ ] Structured-reference spacing is left to T-3394.
- [ ] Every quirk found is written up in docs/excel-semantics/lookup.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Structured-reference text (T-3394).
