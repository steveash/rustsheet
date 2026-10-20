---
id: T-0982
title: "TEXT fraction formats with escaped literals, =/= separators and _ spacers between the parts (211 poi known failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0953, T-2029, T-2030]
---

## Goal
The fraction formats of POI's NumberFormatTests render like Excel: an integer part, a numerator and a
denominator separated by escaped literals (`\:`), quoted or bare literals around the slash (`=/=`),
`_x` spacers and `?`/`#`/`0` placeholders of any mix.

## Context
Evidence: `poi/NumberFormatTests.xlsx` (desktop Excel cached values, Mac Excel 16.0), `Tests!An` =
`=TEXT(Cn, Bn)`, rows A120:A332 except the empty rows A164 and A197: 211 known failures, all
TypeMismatch with actual `#VALUE!`. Values 0, ±0.75, ±1, ±3.75, ±23.75. About 40 format shapes, each on
five values, e.g. (expected):
- `|#\:#/#|`: 0.75 → "|3/4|", -0.75 → "-|3/4|", -1 → "-|1|", 0 → "|0|".
- `|0\:#=/=#|`: -0.75 → "-|0:3=/=4|"; `|#\:0=/=#|`: -1 → "-|1:0=/=1|".
- `|?\:#=/=#|`: -0.75 → "-|  3=/=4|"; `|#\:#=/=?|`: -1 → "-|1      |", 0 → "|0      |".
- `|#_#/#|`: -0.75 → "-|3 /4|", 0 → "|0 /1|"; `|#-#-#\:#/#|`: -0.75 → "-|--3/4|", -1 → "-|--1|".
- `|#\:? ?=/=#|`: 0 → "|:0      |"; `|#\:? ?0#/000`: 0 → "|:0  00/001".
Every format starts with `|`, which the lexer rejects today, so T-0953 (literal `|`, `£`, `¥`) must land
first; after it, re-triage which of the 211 cells still fail and why (the `=/=` separator, the integer
part written with `\:`, blank padding when the fraction part is zero, and the sign in front of the first
literal are the visible rules). Use the `|` side effects T-0953 reports as the starting list.

Code: `crates/rustsheet-engine/src/formatter/` (`lexer.rs`, fraction parsing in `parser.rs`, rendering
in `format.rs`). Record each rule in docs/excel-semantics/number-formats.md with its cells.

Excel-cached evidence from Mac Excel 16.0; if a rule looks platform-specific, check one cell with the
excel-probe skill (decision 0019/0030) before implementing it.

## Acceptance criteria
- [ ] The NumberFormatTests F lines in A120:A332 disappear from eval/baseline/poi.txt with no new F
      lines; any cell left over is listed in the Outcome with its cause and a follow-up task.
- [ ] Rules recorded in number-formats.md citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fraction section itself and the choice of fraction: T-2029 and T-2030 (gap triage 2026-10-19);
  this task adds the literals, `=/=` and spacers on top of them.
- Thousands grouping (T-0952); literal `|`, `£`, `¥` (T-0953).
- The split into smaller PRs if the re-triage after T-0953 finds independent causes: file them as
  separate tasks instead of growing this one.
