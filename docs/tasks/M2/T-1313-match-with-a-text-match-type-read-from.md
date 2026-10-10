---
id: T-1313
title: "MATCH with a text match_type read from a cell is #REF! (gen-m2 04cfd82f54dbc2b2 A81, f9d4df4efdfbcbd5 A14)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH/INDEX return #REF! when match_type is a text value read from a cell (a literal text one stays #VALUE!).

## Context
Rule and evidence: docs/excel-semantics/lookup.md "MATCH descending search skips backward; a text match_type read from a cell is #REF!" (probes/match-descending-argcheck J9-J20). Fixes gen-m2/04cfd82f54dbc2b2 Calc!A81 and f9d4df4efdfbcbd5 Calc!A14. Check how fn_match sees whether the argument was a reference. Untested kinds (numeric text, boolean, blank or number in the cell): probe them first.

## Acceptance criteria
- [ ] probes/match-descending-argcheck J9-J20, A81 and A14 pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
