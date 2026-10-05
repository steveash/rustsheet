---
id: T-0126
title: "Probe sorted MATCH/LOOKUP/HLOOKUP over whole column/row with blank tail"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0212]
---

## Goal
Oracle evidence for approximate-match lookups (MATCH type 1/-1, LOOKUP, HLOOKUP/VLOOKUP sorted) over whole-column/row references with a blank tail, and the engine matches it.

## Context
T-0119 made whole-column/row search vectors value-neutral (`SearchVector`: only the used part is evaluated, the rest is a virtual run of empty cells of full length), so the engine's results are those of a full walk, where each probed empty cell compares as 0/""/FALSE. T-0212 implements Excel's type-skipping binary search (blanks never match, VERIFIED on bounded gen-m2 ranges); this task confirms it carries over to a whole column/row with a blank tail (docs/excel-semantics/lookup.md, "Whole-column / whole-row search vectors: blank tail"). Engine values today: with numbers 10,20,30 in A1:A3 and a,b,c in B1:B3, `MATCH(99,A:A,1)` = 1048576, `VLOOKUP(99,A:B,2)` = 0, `LOOKUP(20,A:A,B:B)` = 0, `HLOOKUP(99,1:2,2)` = 0 (data in A1:C2). Expected per the verified rule: the blank tail is skipped (the "last number" idiom `MATCH(9.99E+307,A:A)`).

Probe cases (each with the data above, and again with an unrelated far cell such as Z100 filled):
- sorted: `MATCH(99,A:A,1)`, `MATCH(99,A:A,-1)`, `MATCH(5,A:A,1)`, `MATCH(9.99E+307,A:A)`, `VLOOKUP(99,A:B,2)`, `HLOOKUP(99,1:2,2)`, `LOOKUP(20,A:A,B:B)`, `LOOKUP(99,A:A,B:B)`, `LOOKUP(30,A1:A3,E:E)`; bounded: `MATCH(99,A1:A10,1)`, `VLOOKUP(99,A1:B10,2)`, a blank in the middle (A2 empty).
- exact on blanks: `VLOOKUP(0,A:B,2,FALSE)`, `HLOOKUP(FALSE,1:2,2,FALSE)`, `VLOOKUP("",A:B,2,FALSE)`, `MATCH(0,A:A,0)`, `XMATCH(0,A:A)`, and a lookup value that is a reference to a blank cell.

## Acceptance criteria
- [ ] Probe cases in eval/suites/probes with oracle values; docs/excel-semantics/lookup.md records the rules (VERIFIED) and the engine matches them, keeping `SearchVector`'s bounded evaluation.
- [ ] The unit tests in test_fn_match/vlookup/hlookup/lookup.rs pin the probed values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
