---
id: T-1280
title: "Probe EXACT, REPLACE, PROPER, CLEAN and CODE edge cases (evidence gap: 4-24 Excel cells each)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
EXACT, REPLACE, PROPER, CLEAN and CODE each have at least 25 Excel-valued cells in eval, covering their scalar edge cases, and any engine mismatch the probe finds is fixed (if local to the function) or filed as a task.

## Context
Gap triage 2026-10-09 (decision 0031): report.md "Excel evidence per function" lists these M5 text functions below the 25-cell threshold: EXACT 24 Excel cells (prior 0.5), REPLACE 11 (0.5), PROPER 15 (0.3), CLEAN 4 (0.3), CODE 4 (0.3). The existing evidence is array broadcasting (probes/indirect-text-broadcast, text.md "Text functions broadcast over array arguments") and argument modes (probes/text-argument-modes, text.md "Text function argument modes"); the per-function scalar rules are unprobed.

Write one probe case `eval/suites/probes/cases/text-scalar-edges.case.json` (excel-probe skill). Suggested cells:
- EXACT: `EXACT("a","A")`, `EXACT("a ","a")`, `EXACT(1,"1")`, `EXACT(TRUE,"TRUE")`, `EXACT("","")`, `EXACT(<blank>,"")`, `EXACT(1.0,"1")`, `EXACT(0.1+0.2,"0.3")`, `EXACT("é","é")` with the second built as `"e"&UNICHAR(769)` (composed vs decomposed), `EXACT(#N/A,"a")`.
- REPLACE: `REPLACE("abcdef",3,2,"XY")`, `(..,1,0,"Z")` (insert), `(..,7,1,"Z")` (append at end), `(..,10,1,"Z")` (start past end), `(..,2,100,"")`, `(..,0,1,"Z")` and `(..,1,-1,"Z")` (errors), `(..,2.9,1.9,"Z")` (truncation), `REPLACE(12345,2,3,0)` (numbers), `REPLACE(TRUE,1,1,"x")`, `REPLACE("abc","2","1","x")`.
- PROPER: `PROPER("hello world")`, `"o'neil"`, `"mary-jane"`, `"abc123def"`, `"ÉCOLE élève"`, `"x.y z_w"`, `"ALL CAPS"`, `PROPER(1.5)`, `PROPER(TRUE)`, `PROPER("")`.
- CLEAN: `CLEAN(CHAR(9)&"a"&CHAR(10))`, `CLEAN("a"&CHAR(31)&"b")`, `CLEAN("a"&CHAR(127)&"b")`, `CLEAN("a"&UNICHAR(129)&"b")`, `CLEAN("a"&UNICHAR(160)&"b")`, `CLEAN("a"&UNICHAR(8203)&"b")`, `LEN(CLEAN(CHAR(13)&CHAR(10)))`, `CLEAN(123)`.
- CODE: `CODE("A")`, `CODE("abc")`, `CODE("")`, `CODE(" ")`, `CODE("é")`, `CODE("€")`, `CODE(UNICHAR(8364))`, `CODE(UNICHAR(128512))`, `CODE(1)`, `CODE(TRUE)`, `CODE(#DIV/0!)`.
Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. CODE above 127 depends on the code page; record what excel-graph returns (it is the en-US web answer; desktop questions go to M14, decision 0039).

Code: `fn_exact` (functions/text/common.rs), `fn_replace` / `replace_element` and `fn_proper` (functions/text/string_format.rs), `fn_clean` and `fn_code` (functions/text/char_code.rs), all under crates/rustsheet-engine/src/.

## Acceptance criteria
- [ ] `probes/text-scalar-edges` is committed with its excel-graph oracle sidecar, and report.md's evidence table shows each of the five functions with 25 or more Excel cells.
- [ ] docs/excel-semantics/text.md records the results (VERIFIED, citing cells), including PROPER's word-boundary rule, the characters CLEAN removes and CODE's result for non-ASCII text.
- [ ] Every failing probe cell is either fixed in this PR (only if local to the function) or filed as a task naming the cells; `make eval-baseline` shows no known failures outside the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- REPLACE/MID start_num and num_chars bounds (huge values; T-1258 on branch agent/harness/T-1239-gen-m6-suite) and UTF-16 position counting (T-0249).
- Array and range lifting (already evidenced).

## Outcome
Added `probes/text-scalar-edges` (63 excel-graph cells). Two mismatches found and fixed in place: CLEAN also
removes the C1 controls 128-159, and CODE of a character outside Windows-1252 is 63, not `#VALUE!`. Everything
else matched. Recorded in docs/excel-semantics/text.md ("CLEAN and CODE scalar edges"). Eval: probes known
failures 2853 -> 2853, one new case, all cells pass.
