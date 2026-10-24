---
id: T-2467
title: "Probe TEXTJOIN's lifted edges: argument cap, legacy range delimiter, element vs text error, blank in a range ignore_empty"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles four TEXTJOIN edges that T-2318 left unprobed, and docs/excel-semantics/arrays.md records each answer.

## Context
Follow-up from the T-2318 reviews (PR 1596). T-2318 put TEXTJOIN in the SPEC-0011 lifting table
(`functions/lift/kinds/text.rs`, `[WholeByCaller, Lift]`, repeat `WholeByCaller`, `max: 255`), backed by
probes/textjoin-ignore-empty-lift (132 cells). Unprobed, per arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted
arguments" ("Ours:" line):
1. The argument cap: 254, 255 and 256 arguments (Excel's documentation says 252 texts; ours accepts 255).
2. A legacy (non-dynamic-array) formula with a range delimiter, e.g. `=TEXTJOIN(A1:A3,TRUE,B1:B3)`: whether the
   range is kept whole (ours, `Signature::Vector`) or implicitly intersected.
3. An error element of `ignore_empty` beside a text error: `TEXTJOIN(",",{TRUE;#DIV/0!},#N/A,"a")` (ours: the
   element's error in its place, `#N/A` elsewhere).
4. A blank cell inside a range `ignore_empty` (ours reads it as FALSE).

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) covers the four items; its cells are filled and gated.
- [ ] arrays.md states each answer with its evidence, and any mismatch with ours gets a `campaign: visible` task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing TEXTJOIN: a mismatch gets its own task.
