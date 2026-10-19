---
id: T-1360
title: "REGEXTEST, REGEXEXTRACT and REGEXREPLACE accept their optional case_sensitivity, return_mode and occurrence arguments (#ERROR! today)"
milestone: M7
lane: functions-text
status: in-progress
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

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3462856259a37891 Calc!A39 `=(("a"&TRIM("?b"))&SORTBY(Data!B1:E12,REGEXREPLACE(Data!B10,"(\w)(\w)","<$0>",Data!D12,1),..))`: Excel `#VALUE!`, engine `#ERROR!`.
- gen-m9/3ec66461400faf2f Calc!A35 `=RIGHT(REGEXEXTRACT((Data!$D$1&""),"(\w)(\w)",1,1),7)` ("0.1", no match): Excel `#N/A`, engine `#ERROR!`.
- gen-m9/65faaeb48419087d Calc!A31 `=TEXT(SHEET(),(("x~*"&Data!E6)&REGEXREPLACE("1","(\w)(\w)","$1",0.1,1)))`: Excel `#VALUE!`, engine `#ERROR!`.
- gen-m9/adb3fd63c7368166 Calc!A1 `=REGEXTEST(CELL("type",Data!E1),"a",0)` ("l"): Excel FALSE, engine `#ERROR!`.
- gen-m9/adb3fd63c7368166 Calc!A8 `=REGEXREPLACE(Data!$D7,CONCATENATE(Data!$C4),"-",-1)` (D7 = TRUE, C4 blank, occurrence -1): Excel "TRUE-", engine `#ERROR!`.
- gen-m9/adb3fd63c7368166 Calc!A30 `=REGEXREPLACE(JIS(Data!A$5),"a|b","",0,0)`: Excel `#NAME?` (JIS is `#NAME?` on the web), engine `#ERROR!`: the arity check must not hide an argument's error.
- gen-m9/d244411dd90f7fbd Calc!A38 `=CONCATENATE(REGEXREPLACE(SUBSTITUTE(Data!C5,Data!A$12,Data!B6),CONCAT("ABC",Data!$D8),"a",-1,Data!$D7),Data!D6)`: Excel "abc0.000000000000001", engine `#ERROR!`.

## Acceptance criteria
- [ ] gen-m7/8ed8a680e9794b77 A1 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] An excel-graph probe case covers the optional arguments listed above and passes; text.md records the rules (VERIFIED).
- [ ] gen-m9/3462856259a37891 A39, 3ec66461400faf2f A35, 65faaeb48419087d A31, adb3fd63c7368166 A1, A8 and A30, and d244411dd90f7fbd A38 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

More evidence (T-1824, probes/textsplit-lift-guesses): A53 `REGEXEXTRACT({"a1b2";"c3d4"},"([a-z])([0-9])",2)` is "a","c" in Excel (ours "a1","c3"); A55 `REGEXEXTRACT("a1B2","b",0,{0;1})` is `#N/A` (ours `#ERROR!`, the fourth argument).
