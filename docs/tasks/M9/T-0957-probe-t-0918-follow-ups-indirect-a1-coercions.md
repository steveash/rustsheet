---
id: T-0957
title: "Probe T-0918 follow-ups: INDIRECT a1 coercions and text-function array broadcasting"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for INDIRECT's `a1` argument coercions and for array broadcasting of the text functions that only have IronCalc-ported behavior.

## Context
T-0918 review follow-up (carried in T-0920): oracle-confirm INDIRECT's `a1` coercions (`0`,
`""`, `"x"`) and text-function array broadcasting for EXACT/FIND/LEN/REPT/SEARCH/SUBSTITUTE/TRIM/
CLEAN/PROPER/REPLACE (only LEFT/MID/RIGHT have Excel-authored evidence); add a
docs/excel-semantics/ entry for R1C1 offset wrapping. Use the excel-probe skill.

## Acceptance criteria
- [ ] Probe cases with oracle values; docs/excel-semantics entries; follow-up tasks filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
