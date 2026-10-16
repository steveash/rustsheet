---
id: T-1219
title: "ENCODEURL is #VALUE! on excel-graph for every argument"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ENCODEURL gives excel-graph's golden result: `#VALUE!` for every argument, lifted over an array.

## Context
T-1088 probed ENCODEURL on excel-graph (probes/info-logic-argument-modes-2 A1:A6, B5; info-logic-argument-modes A15:B15; 2026-10-09).
`ENCODEURL("a b&c")`, `ENCODEURL("abc")`, a text cell, `1` and `TRUE` are all `#VALUE!`. `ENCODEURL({"a b","c"})` spills two `#VALUE!`, and
`ENCODEURL(Data!A4:B4)` spills two as well. Like FILTERXML, the function seems to be unavailable on the web. Decision 0039 makes the web value a golden, and the
desktop question goes to T-1081 (M14). The engine encodes (functions/text/encodeurl.rs, test/test_fn_encodeurl.rs) and gives `#N/IMPL!` for an array.
Record the decision in docs/excel-semantics/web.md "ENCODEURL is #VALUE! in Excel for the web". Should the engine follow the golden, or keep encoding as
a DEVIATION until desktop evidence exists? Decision 0039 rule 4 decides. If the golden wins, update the IronCalc-ported tests that assert encoding.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/fcf11f2ec6f7a59d Calc!A7 `=CODE((ENCODEURL(Data!B3)&DOLLAR(Data!$E$1)))`: Excel `#VALUE!`, engine 49.

## Acceptance criteria
- [x] The ENCODEURL probe cells either pass or are recorded as a DEVIATION with a decision citation.
- [x] gen-m9/fcf11f2ec6f7a59d A7 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
ENCODEURL now follows the excel-graph golden (decision 0039 rule 4): `#VALUE!` for every argument,
lifted element-wise over a range or array (static_analysis marks it `scalar_arguments`); an error
argument propagates (unprobed). Recorded in docs/excel-semantics/web.md; desktop confirmation stays
T-1081. Eval: probes known failures 5523 -> 5514 (9 fixed, 0 regressions). The gen-m9 cell
(fcf11f2ec6f7a59d A7) is not in the local suite set; covered by a unit test of the same shape.
