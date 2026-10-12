---
id: T-1313
title: "MATCH with a text match_type read from a cell is #REF! (gen-m2 04cfd82f54dbc2b2 A81, f9d4df4efdfbcbd5 A14)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH/INDEX return #REF! when match_type is a text value read from a cell (a literal text one stays #VALUE!).

## Context
Rule and evidence: docs/excel-semantics/lookup.md "MATCH descending search skips backward; a text match_type read from a cell is #REF!" (probes/match-descending-argcheck J9-J20). Fixes gen-m2/04cfd82f54dbc2b2 Calc!A81 and f9d4df4efdfbcbd5 Calc!A14. Check how fn_match sees whether the argument was a reference. Untested kinds (numeric text, boolean, blank or number in the cell): probe them first.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/6db767d720c5b55a Calc!A5 `=(MATCH(SUMPRODUCT(Data!B8,Data!$C3),Data!B3:B4,IFS(TRUE,Data!D$3))=3.14159)` with D3 = "x" (match_type is text from a cell, through IFS): Excel `#REF!`, engine `#VALUE!`.

## Acceptance criteria
- [x] probes/match-descending-argcheck J9-J20, A81 and A14 pass; baseline updated.
- [x] gen-m7/6db767d720c5b55a A5 (text match_type through IFS): moved to T-1399; IFS has no reference pass-through.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`match_type_arg` reads match_type in reference context; non-numeric text from a cell is #REF!, literal text stays #VALUE!. probes/match-descending-argcheck 20/20, gen-m2 04cfd82f54dbc2b2 A81 and f9d4df4efdfbcbd5 A14 pass (gen-m2 known failures 28 -> 26, probes 2888 -> 2883). A5 deferred to T-1399.
