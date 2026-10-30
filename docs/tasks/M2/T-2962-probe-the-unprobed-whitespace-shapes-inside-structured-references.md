---
id: T-2962
title: "Probe the unprobed whitespace shapes inside structured references (padded lone specifier, line feed after the comma, padded # and @ starts)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2801]
---

## Goal
Excel evidence for the whitespace shapes inside a structured reference that T-2801 left to our own choice, recorded in
syntax.md "Whitespace beside a missing argument is kept; a structured reference keeps some and normalizes the rest".

## Context
T-2801 implemented the probed shapes (probes/t2786-whitespace-structured-refs-runs F4:F15). syntax.md marks these
choices as ours and unprobed:
- a run around a lone specifier keeps its outer brackets: `=T[ [#Data] ]` and `=T[ [#Headers] ]` print as typed;
- a line feed after the comma prints as one space: `=T[[#Data],` LF `[Col]]` (the one data point used spaces only);
- `=T[ #Data ]` and `=T[ @Col ]` keep their pre-T-2801 reading, a column whose name has the spaces;
- a stored `T[ Col ]` reads back as column `Col` (loading is also T-2800's).
Also worth one cell: `=T[ [Col,1] ]` with a header that needs brackets (`Col,1`), since ours prints `T[ Col,1 ]`.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, a table `T` as in t2786) has oracle values for the shapes above: FORMULATEXT
      and the value, or the refusal Graph gives.
- [ ] syntax.md records the answers, and a follow-up engine task is filed for any that differ from ours.
- [ ] `make check` green; reviewers approve.

## Out of scope
- This-row shapes and loading hand-written stored text (T-2800).
