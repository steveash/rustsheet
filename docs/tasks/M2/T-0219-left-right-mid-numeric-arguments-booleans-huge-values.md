---
id: T-0219
title: "LEFT/RIGHT/MID numeric arguments: booleans, huge values and near-integers (9 gen-m2 known failures)"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
LEFT, RIGHT and MID coerce their numeric arguments the way Excel does. Booleans count as 0/1. Very large
start_num/num_chars give `#VALUE!`. A num_chars a hair below an integer counts as that integer.

## Context
gen-m2 (T-0203): 9 failing cells. Evidence (excel-graph):
- Booleans: `gen-m2/ef5c55a81783b7d5` Calc!A40 `=MID((Data!E$4&(Data!A10&"x~*")),Data!B$4,5)` (B4 = TRUE):
  expected "0.1x~", actual `#VALUE!`. `gen-m2/d04d501154afc771` Calc!A19 `UPPER(RIGHT(Data!A11,Data!B1))`
  (B1 = FALSE gives ""): expected TRUE overall, actual `#VALUE!`.
- Huge arguments are `#VALUE!`: `gen-m2/78bc88199fc96650` Calc!A345 `MID(Data!D12,2,1000000000000000.0)`;
  `gen-m2/d0c29dd5c530e28c` Calc!A269 `MID("ü",1000000000000000.0,4)`; `gen-m2/e8493fafa2a5378e` Calc!A337
  `MID("1.5",SUM(...)=1000000246913575.8,7)`; `gen-m2/ef5c55a81783b7d5` Calc!A299 `MID(Data!$B4,Data!D8,5)`
  (D8 = 1E+15); `gen-m2/caa981b0753f455c` Calc!A357 (start 1E+15); `gen-m2/b4392a6514a36ade` Calc!A217
  (num_chars 1000000000000041.9). The engine returns "" or the text. Probe the threshold (32767? 2^31?)
  with the excel-probe skill.
- Near-integer: `gen-m2/b4392a6514a36ade` Calc!A14 `LEFT((Data!C7&"?b"),(Data!C10-Data!E$10))` with
  num_chars = 0.999999999999999 (engine value): expected "?" (1 char), actual "". Probe how near counts as
  an integer (cf. docs/excel-semantics/numbers.md "Comparison tolerance").

Code: `crates/rustsheet-engine/src/functions/text/` (LEFT/RIGHT/MID argument parsing). One helper for
"text-function count argument".

## Acceptance criteria
- [ ] Probe case(s) pin the huge-value threshold and the near-integer rule. docs/excel-semantics/ entry added.
- [ ] The 9 cells above disappear from eval/baseline/gen-m2.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number-to-text of the first argument (T-0107).
- Boolean and numeric-text count arguments (gen-m2 ef5c55a8 A40, d04d5011 A19): T-0256 owns them.
