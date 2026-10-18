---
id: T-1347
title: "TEXTSPLIT of empty text is #VALUE! (gen-m7 3a62e0950c1074fd A5)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT of empty text is `#VALUE!`, as in Excel, instead of returning "".

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/3a62e0950c1074fd Calc!A5 `=(Data!B5>TEXTSPLIT("","a",CONCAT(Data!D9)))` with B5 blank and D9 = "TRUE":
Excel `#VALUE!`, engine FALSE (the engine's `TEXTSPLIT("","a")` is "", and blank > "" is FALSE). A comparison cannot make
`#VALUE!` from a blank and text, so TEXTSPLIT itself is `#VALUE!` (docs/excel-semantics/text.md "TEXTSPLIT of empty text").
Code: `fn_textsplit` in crates/rustsheet-engine/src/functions/text/textsplit.rs. Probe with the excel-probe skill whether a
blank cell as text (`TEXTSPLIT(A1,",")`), `TEXTSPLIT("",",")` with no row delimiter and `TEXTSPLIT("","")` behave the same, and
match those too.

## Acceptance criteria
- [ ] gen-m7/3a62e0950c1074fd A5 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] The probe cells pass; text.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`textsplit_element` returns `#VALUE!` for empty text, after the other arguments are checked. New probe
probes/textsplit-empty-text (19 cells, excel-graph, all pass): literal, blank-cell and empty-string text, with and
without a row delimiter, `ignore_empty`, `match_mode` and `pad_with`, an empty delimiter, and `ISERROR`, an error-valued argument (it wins) and array/range text (each empty element is `#VALUE!`); every empty-text form is
`#VALUE!`. gen-m7/3a62e0950c1074fd now passes in full (39/39). Eval delta: +19 passing probe cells, no new known failures;
docs/excel-semantics/text.md entry is VERIFIED.
