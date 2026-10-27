---
id: T-2942
title: "Keep whitespace beside a missing argument (SPEC-0023)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Whitespace beside a missing argument prints and saves as Excel does.

## Context
T-2786 found Excel keeps a run between commas or after the last comma (`=IF(1, ,2)`, `=SUM(1, )`, `=NOW( )`, `=SUM(, 1)`), where SPEC-0023 drops a run before a comma or `)`. A saved `<f>` writes a line feed as raw CR LF bytes, which read back as LF. See syntax.md "Whitespace beside a missing argument is kept" and xlsx-storage.md "Typed whitespace is saved"; probes/t2786-whitespace-structured-refs-runs F/I cells. `=SUM (1)` and `=SUM( )` are refused by Graph (not a parse requirement).

## Acceptance criteria
- [ ] The I1:I13 FORMULATEXT cells of probes/t2786-whitespace-structured-refs-runs pass.
- [ ] SPEC-0023 "Where whitespace attaches" updated; a kept line feed round-trips as LF.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Structured references (T-2801).
