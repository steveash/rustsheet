---
id: T-1861
title: "MATCH with an array lookup_value of empty strings over a concatenated range (corpus-bench 530-28)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
corpus-bench/530-28 `Sheet1!P8` and `Q8` match Excel's cached "".

## Context
Split from T-1791. CSE `{=INDEX($A:$A,SMALL(IF(($B$2:$B$176&$C$2:$C$176=$E2&$F2)*MATCH($A$2:$A$176&"",$A$2:$A$176&"",)=ROW($1:$175),ROW($2:$176),4^8),COLUMN(J$1)))&""}`: expected `""`, engine `#VALUE!`. In a unit test with A2:A6 blank, `{=MATCH($A$2:$A$6&"",$A$2:$A$6&"",)}` is `#N/A` in the engine (Excel: each "" matches position 1), so suspect MATCH with an array lookup_value, an empty-string lookup, or the omitted match_type (`,)` = 0). `INDEX($A:$A,4^8)&""` and `SMALL(IF(...=ROW($1:$3),ROW($2:$4),4^8),2)` already work. Probe `MATCH("",{"";""},0)`, `MATCH({"a";""},{"";"a"},)` and the lifted form before fixing. Inspect with `./target/release/rustsheet-eval show corpus-bench/530-28/1_530-28_input.xlsx` (fetch the suite first).

## Acceptance criteria
- [ ] corpus-bench/530-28 P8:Q8 pass, with a unit or spec test for the cause; any quirk goes into docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
