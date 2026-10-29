---
id: T-3059
title: "IM* inumber: probe leading spaces before NBSP/LF/CR/U+3000, end whitespace then trailing spaces, \"1E$3\" and \"12$3i\""
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every inumber shape engineering.md "How an inumber is read" lists as unprobed after T-2641 has Excel evidence, and ours matches it (or a follow-up owns the gap).

## Context
T-2641 (probes/eng-inumber-currency-shapes, excel-graph) found a tab, NBSP, LF, CR or U+3000 at either end is `#NUM!`, but past leading spaces only tab is probed (A26, " "&CHAR(9)&"5"). Unprobed, ours in brackets: " "&UNICHAR(160)&"5", " "&CHAR(10)&"5", " "&CHAR(13)&"5", " "&UNICHAR(12288)&"5" [`#NUM!`]; "5"&CHAR(9)&" ", "3+4i"&UNICHAR(160)&" " [read]; "1E$3", "12$3i" [`#NUM!`]; a tab inside the text ("3"&CHAR(9)&"+4i").
Code: `complex_of` (complex/mod.rs) and `strip_currency_symbols` (complex/reading.rs).

## Acceptance criteria
- [ ] A probe case with these cells is oracle-filled (excel-graph); engineering.md moves each shape out of Unprobed.
- [ ] Any shape where ours differs is fixed here (size S) or filed as a functions-engineering task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date-dependent cells.
