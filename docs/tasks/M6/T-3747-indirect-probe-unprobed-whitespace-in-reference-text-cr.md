---
id: T-3747
title: "INDIRECT: probe unprobed whitespace in reference text (CR, inner LF, spaces inside $A$1 and R[1]C, names)"
milestone: M6
lane: functions-lookup
status: in-progress
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
- [ ] A probe case covers the shapes above in A1 and R1C1 style, with oracle values.
- [ ] The engine matches every probed cell; lookup.md's entry lists the new shapes and drops them from "Not probed".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Structured-reference text (T-3394).
