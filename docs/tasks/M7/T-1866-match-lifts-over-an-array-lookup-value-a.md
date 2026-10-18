---
id: T-1866
title: "MATCH lifts over an array lookup_value (a 1x1 array included)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH with an array lookup_value (an array constant, a function result such as TEXTSPLIT, or a multi-cell range) returns one position per element, as Excel does, instead of `#N/A`.

## Context
Found by T-1361. gen-m7/4629e931955aa4b8 Calc!A19 `=(MATCH(TEXTSPLIT("#N/A",Data!$B4,Data!B11:B12),Data!E8:E10)-LOG10(0))`
(E8:E10 = 0.1, " x ", 0.2): Excel `#NUM!` (MATCH is a number, so LOG10(0)'s `#NUM!` wins), ours `#N/A` since T-1361 (TEXTSPLIT
now gives {"#N/A"}). In the engine `MATCH({"x"},E8:E10)` and `MATCH({"#N/A"},E8:E10)` are `#N/A` while `MATCH("x",E8:E10)`
is 2. docs/excel-semantics/lookup.md "A reference-valued lookup_value is looked up by its cell's value" says a multi-cell
lookup_value lifts in XMATCH and LOOKUP (T-1241) and names MATCH as T-0270's, which landed only VLOOKUP/HLOOKUP. Code:
`Model::lookup_value_arg` (functions/lookup_and_reference/mod.rs) and MATCH; reuse XMATCH's lift, do not copy it. Probe
`MATCH({"x";0.2},E8:E10)`, `MATCH({"x"},E8:E10)`, `ROWS(MATCH(E8:E9,E8:E10,0))` and a match_type array first if lookup.md
has no evidence.

## Acceptance criteria
- [ ] gen-m7/4629e931955aa4b8 A19 passes in `./target/release/rustsheet-eval show gen-m7/4629e931955aa4b8.case.json`, no new failing gen-m7 cell.
- [ ] Unit tests for a 1x1 array, a 2x1 array and a multi-cell range lookup_value; lookup.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
