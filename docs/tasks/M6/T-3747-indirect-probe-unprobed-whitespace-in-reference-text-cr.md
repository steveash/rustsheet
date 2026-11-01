---
id: T-3747
title: "INDIRECT: probe unprobed whitespace in reference text (CR, inner LF, spaces inside $A$1 and R[1]C, names)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
INDIRECT matches Excel for whitespace shapes T-3656 left unprobed. `indirect_reference_text` currently refuses them by generalising from tab and NBSP.

## Context
T-3656 (probes/t3656-indirect-spaces-reversed; lookup.md "INDIRECT reference text: where spaces are allowed")
probed leading/trailing space, LF, tab and NBSP, and spaces around `:` and `!`. Not probed, and `#REF!` in the
engine today: a carriage return (leading, trailing, `CHAR(13)&CHAR(10)`), a line feed that is not trailing
(`"A1:"&CHAR(10)&"B2"`), spaces inside a reference (`"$A $1"`, `"A 1"`, `"R[ 1]C"`, `"R1 C1"`), other Unicode
spaces (`UNICHAR(8195)`), and a defined name with trailing spaces (`"MyName "`; the name fallback sees the text as
written). Write a probe case (excel-graph) like the T-3656 one (SUM/ROWS/COLUMNS/CELL("address") per text), then
change `indirect_reference_text` (engine expressions/utils) only where the oracle differs.

## Acceptance criteria
- [x] A probe case covers the shapes above in A1 and R1C1 style, with oracle values.
- [x] The engine matches every probed cell; lookup.md's entry lists the new shapes and drops them from "Not probed".
- [x] `make check` green; reviewers approve.

## Out of scope
- Structured-reference text (T-3394).

## Outcome
Probed (excel-graph, 2026-11-01): `probes/t3747-indirect-whitespace` (100 cells) and
`probes/t3747-indirect-name-whitespace` (8 cells). The engine already refused CR, leading LF, Unicode
spaces and spaces inside references correctly. Two differences fixed: a line feed next to the range colon
resolves like a space (`"A1:"&CHAR(10)&"B2"`), and a defined name may end in a line feed. lookup.md updated
(the "Not probed" shapes are now listed with their results). Eval: 2 new cases, 0 known failures, baseline
updated.
