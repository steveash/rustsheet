---
id: T-1360
title: "REGEXTEST, REGEXEXTRACT and REGEXREPLACE accept their optional case_sensitivity, return_mode and occurrence arguments (#ERROR! today)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
REGEXTEST(text, pattern, [case_sensitivity]), REGEXEXTRACT(text, pattern, [return_mode], [case_sensitivity]) and REGEXREPLACE(text, pattern, replacement, [occurrence], [case_sensitivity]) accept their optional arguments and compute what Excel does, instead of `#ERROR!`.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11):
- gen-m7/8ed8a680e9794b77 Calc!A1 `=REGEXTEST(ADDRESS(18,25,3,VLOOKUP(1.005,Data!D4:E6,1,TRUE),"Data"),"[a-z]+",1)`: Excel TRUE,
  engine `#ERROR!`. The ADDRESS part is fine ("Data!$Y18"); `REGEXTEST("abc","[a-z]+",1)` and `(..,0)` are `#ERROR!` in the engine.
- gen-m7/6db767d720c5b55a Calc!A29 `=(REGEXREPLACE(MAKEARRAY(3,3,_xleta.AVERAGE),"a|b","",2,1)&..)`: Excel `#SPILL!`, engine `#ERROR!`
  (also needs T-1273 for `_xleta.AVERAGE`).
The arity checks reject the optional arguments: `args_signature_scalars(arg_count, 2, 0)` / `(3, 0)` / `(2, 1)` in
crates/rustsheet-engine/src/expressions/parser/static_analysis.rs and `args.len()` checks in
crates/rustsheet-engine/src/functions/text/regex_functions.rs (REGEXEXTRACT takes 3 today, its third argument read as a
`return_all` flag). Opus: only the two cells above are Excel evidence; probe the semantics first with the excel-probe skill
(case_sensitivity 0/1 and invalid values, REGEXEXTRACT return_mode 0/1/2 with and without capture groups, REGEXREPLACE
occurrence 0, 2, -1 and beyond the match count, non-integer and text values) and record them in docs/excel-semantics/text.md.
Lifting over a range or array is T-1272.

## Acceptance criteria
- [ ] gen-m7/8ed8a680e9794b77 A1 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] An excel-graph probe case covers the optional arguments listed above and passes; text.md records the rules (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
