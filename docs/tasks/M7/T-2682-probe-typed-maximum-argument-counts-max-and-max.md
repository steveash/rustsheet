---
id: T-2682
title: "Probe typed maximum argument counts (max and max+1) of repeating and multi-form functions"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Typed entry's maximum argument counts (`functions/kinds/arg_counts.txt`) are evidenced, not inferred, for the functions
where a wrong maximum would refuse a formula Excel takes.

## Context
T-2632 made typed entry refuse a call above the catalog's maximum for every function. Only `=ABS(1,2)` is probed
(Graph PATCH 400); the other maxima come from Microsoft's syntax lines (255 when a line repeats) and are UNVERIFIED in
docs/excel-semantics/syntax.md "Minimum argument counts". Probe with typed `entered` cells (excel-probe skill; a refused
input fails the fill with 400 `InvalidArgument` and cannot be committed, so record refusals in syntax.md by date) one call
at the maximum and one past it for: a fixed-arity function with optional arguments (XLOOKUP 6/7, IF 3/4), INDEX's
reference form (4/5), a 255-repeating function (SUM with 255/256 arguments, TEXTJOIN 254/255 per Microsoft's 252 texts),
and off-cycle counts of repeating functions outside the lifting table (`=IFS(1,2,3)`, `=SWITCH(1,2)`, `=LET(x,1,y,2)`),
which typed entry still accepts. Too-lenient maxima from unusable syntax lines (T-2632 review): DETECTLANGUAGE and
TRANSLATE get 255 (their notes give `DETECTLANGUAGE(text)` and `TRANSLATE(text, [source], [target])`), and FIND, FINDB,
LEFTB, LENB, MIDB, REPLACEB, RIGHTB and SEARCHB get 255 in the table (the lifting table still caps them); give them cited
`MAX_OVERRIDES`.

## Acceptance criteria
- [x] Each probed count is recorded in syntax.md as accepted (with an oracle sidecar) or refused (Graph 400, dated).
- [x] Where a probe disagrees with arg_counts.txt, fix the derivation in tools/tests/test_catalog_arity.py (or its
      `MAX_OVERRIDES`, cited) and regenerate the table; typed API tests cover the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- Stored (file) forms of the same calls.

## Outcome
Done (Graph PATCH, 2026-10-25). Accepted typed at the maximum, with an oracle sidecar: XLOOKUP 6, IF 3, INDEX's reference
form 4, FIND/FINDB/SEARCHB 3, LEFTB/RIGHTB 2, LENB 1, MIDB 3, REPLACEB 4; also whole IFS and LET pairs and SWITCH 3 and 4
(probes/t2682-typed-max-arg-counts A1:A17). TRANSLATE 3 was accepted as `#BUSY!`; that acceptance is logged but has no
sidecar. DETECTLANGUAGE 1 is UNVERIFIED (it rests on the catalog minimum; only its refusal at 2 is logged). Refused (400 `InvalidArgument`): one past each maximum, IFS 3 and 5, LET 4 and 6, and SWITCH 2.
arg_counts.txt had 255 for the eight text functions, DETECTLANGUAGE and TRANSLATE. They now have cited `MAX_OVERRIDES`.
Typed behaviour does not change for the text functions, which the lifting table already capped, or for DETECTLANGUAGE
and TRANSLATE, which are not implemented (`#NAME?`). Typed entry did accept off-cycle IFS and LET calls, and
`arg_counts::takes_count` now refuses them (`PAIRED`). `MAX_OVERRIDES` now replaces the derived maximum instead of only widening it. Not probed: SUM 255/256 and TEXTJOIN
254/255. The oracle's entered cells are custom properties of at most 255 characters, so these are T-2709.
