---
id: T-1533
title: "CHOOSE with an index array over a range argument broadcasts the range"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1487]
---

## Goal
CHOOSE lifted over an index array reads a range argument as Excel does: `ROWS(CHOOSE({1,2},'[other.xlsx]Sheet1'!A1:B3,A1))`
is 3 (we give 1, because `choose_element` reduces each picked argument to one value by implicit
intersection).

## Context
Known failure probes/external-ref-no-link-ifchoose Calc!A32 (excel-graph). Excel apparently
broadcasts the picked ranges elementwise, like IF over an array condition (`if_array` in
functions/logical/mod.rs), instead of implicitly intersecting them. First probe local ranges
(excel-probe): `ROWS(CHOOSE({1,2},A1:B3,C1))`, `=CHOOSE({1,2},A1:A3,B1:B3)` (spill shape and
values), `=CHOOSE({1;2},A1:B1,C1)`, `SUM(CHOOSE({1,2},A1:A3,B1:B3))`, then implement in
`choose_pick`/`choose_element` (functions/lookup_and_reference/mod.rs) and record the rule in
docs/excel-semantics/ (arrays.md or logic.md).

## Acceptance criteria
- [ ] probes/external-ref-no-link-ifchoose A32 leaves eval/baseline/probes.txt, and the local
      probes pass.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CHOOSECOLS/CHOOSEROWS.
