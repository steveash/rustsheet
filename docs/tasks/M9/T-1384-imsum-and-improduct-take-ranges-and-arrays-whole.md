---
id: T-1384
title: "IMSUM and IMPRODUCT take ranges and arrays whole; IM* functions refuse booleans (probes/eng-argument-modes)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSUM and IMPRODUCT take a range or array argument whole, with Excel's rules for what is inside it, and
the IM* functions refuse a boolean argument with `#VALUE!`.

## Context
SPEC-0011 (docs/specs/SPEC-0011-scalar-argument-lifting.md) makes IMSUM's and IMPRODUCT's inumbers `WholeByCaller`: the lifting driver never lifts them (T-1383 enters them in the table). This task's rules decide how they are folded.

Evidence: probes/eng-argument-modes A60:A83 and A89 (excel-graph, 2026-10-11, T-1085), docs/excel-semantics/
engineering.md "Engineering function argument modes". The data is M1 "1+2i", M2 3, M3 blank, M4 TRUE,
M5 "x" and M6 "2-i". Excel's rules:
- A range or array literal is taken whole. `IMSUM(M1:M2)` is "4+2i" and `IMSUM({"1+2i","3"})` is "4+2i".
  `ROWS(IMSUM({"1+2i";"3"}))` is 1, because it does not lift.
- A blank cell is 0. `IMPRODUCT(M1:M3)` is "0", and `IMSUM(M1:M3)` is "4+2i".
- A boolean is `#VALUE!`, whether it is in a cell (`IMSUM(M1:M2,M4)`) or typed (`IMSUM("1+2i",TRUE)`).
  So is `IMABS(TRUE)`.
- Non-complex text is `#NUM!`. `IMSUM(M1:M2,M5)` and `IMSUM(M5:M6)` give `#NUM!`. `IMSUM(M1:M6)`, which
  holds both a boolean and text, gives `#VALUE!`.
- A number cell counts as a real number. `IMSUM("1+2i",M2,"2-i")` is "6+i".

Today we give `#ERROR!` for any range or array, `#VALUE!` where the result should be `#NUM!`, and
`#NUM!` for booleans. Look for an existing helper that walks array arguments before you write one
(`rg -n "fn .*array" crates/rustsheet-engine/src/functions/engineering`).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/b5352ebdb9c5f1a1 Calc!A3 `=SEARCH(PROPER(RIGHT(Data!E8,1)),(IMPRODUCT(Data!C7)&..),7)` (IMPRODUCT of one cell, -0.0001): Excel `#VALUE!` (SEARCH finds no "D"), engine `#ERROR!` from IMPRODUCT.

## Acceptance criteria
- [ ] probes/eng-argument-modes A60:A83 and A89, E42 and E50 leave eval/baseline/probes.txt with no new F
      lines.
- [ ] gen-m9/b5352ebdb9c5f1a1 A3 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting the other engineering functions (T-1383).
