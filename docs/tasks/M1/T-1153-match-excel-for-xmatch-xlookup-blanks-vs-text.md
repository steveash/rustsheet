---
id: T-1153
title: "Match Excel for XMATCH/XLOOKUP blanks vs text and boolean candidates, and a blank lookup value (probes/xmatch-text-bool-equal-runs)"
milestone: M1
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
XMATCH and XLOOKUP agree with Excel for text and boolean lookup values over blank cells, for a blank lookup value, for match_mode 1 binary search over equal runs, and for descending binary search over booleans (Txt!M21, M24).

## Context
Oracle evidence is in probes/xmatch-text-bool-equal-runs and docs/excel-semantics/lookup.md (T-1135). Failing cells:
`Txt!M7` (`XMATCH("a",A1:A6,-1)`: Excel #N/A, engine 2), `Txt!M16` (`XMATCH("z",A1:A6,1)`: Excel 2, engine #N/A),
`Txt!M59` (XLOOKUP of that: Excel 0), `Txt!M21` (`XMATCH(TRUE,B1:B6,-1,-2)`: Excel 5, engine 3),
`Blk!M19`, `Blk!M20` (XLOOKUP of a blank lookup value against a 0 or `""` candidate: Excel "nf", engine matches).
Also `Txt!M24`, `Txt!M51`, `Blk!M1-M5`, `Blk!M7`, `Blk!M9`, `Blk!M16`, `Blk!M17`, `Blk!M21`, `Blk!M22` (a blank lookup value
matches a 0, `""` or FALSE candidate in the engine) and `Run!M5`, `M11`, `M17`, `M35`, `M51` (match_mode 1 with search_mode 2: Excel returns the run's first
cell, the engine the second). In Excel a blank candidate is larger than any text (and, in binary search, any boolean), as it is larger than any number; `linear_scan`
(functions/lookup_and_reference/xmatch.rs) models it as empty text.

## Acceptance criteria
- [ ] Every probes/xmatch-text-bool-equal-runs cell passes (24 baselined failures, listed in `make eval` output) and is removed from the baseline; the rest of the probe case keeps passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing XMATCH/XLOOKUP behavior beyond the cells of probes/xmatch-text-bool-equal-runs.
