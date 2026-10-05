---
id: T-0240
title: "Text concatenation (CONCATENATE and &) truncates at 32767 characters"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Concatenating text with CONCATENATE or `&` past 32,767 characters gives the first 32,767
characters, as Excel does, and `&` stops growing past that bound (directive 5).

## Context
probes/resource-limits (excel-graph, 2026-10-05, T-0012): C9 `LEN(CONCATENATE(REPT("a",32767),"b"))`
= 32767 (we give `#VALUE!`), C10 `LEN(REPT("a",32767)&"b")` = 32767 (we give 32768: `&` has no
bound at all). C11 `LEN(REPT("a",32768))` = `#VALUE!` already passes. Code: `MAX_TEXT_CHARS` in
`crates/rustsheet-engine/src/functions/text/concatenate.rs` (its doc comment claims Excel returns
`#VALUE!`, which the probe disproves) and the `&` operator in `arithmetic.rs`/evaluation. Probe
CONCAT and TEXTJOIN past the limit before changing them (they may be `#VALUE!` instead).
docs/excel-semantics/coercion.md "Text longer than 32,767 characters".

The evidence is excel-graph (web) only; confirm truncation with the excel-desktop backend first
if it is available (decision 0019), and record a CONFLICTING entry if desktop differs.

## Acceptance criteria
- [ ] probes/resource-limits C9 and C10 disappear from eval/baseline/probes.txt with no new F lines.
- [ ] Share one truncation helper between CONCATENATE and `&` (no copy).
- [ ] `make check` green; reviewers approve.

## Out of scope
- CONCAT/TEXTJOIN unless the new probe cells show they truncate too.
