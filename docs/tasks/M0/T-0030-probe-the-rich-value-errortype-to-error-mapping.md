---
id: T-0030
title: "Probe the rich-value errorType to error mapping (ERROR.TYPE of each)"
milestone: M0
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every rich-value `errorType` that Excel writes is mapped to its error by oracle evidence, so `rustsheet-eval snapshot` stops marking such cells `#UNDECODED!`.

## Context
`crates/rustsheet-eval/src/xlsx_snapshot.rs::rich_value_errors` decodes only errorType 8 (`#SPILL!`) and 13 (`#CALC!`), evidenced by ironcalc-calc/ERROR.TYPE.xlsx (errorType = ERROR.TYPE - 1 there). Anything else becomes `#UNDECODED!` and is left unscored (T-0203 review). Known case: poi/link-external-workbook-b.xlsx Sheet0!A1 (uncached external reference), errorType 10 / subType 2 in the excel-graph result; by the pattern this may be `#BLOCKED!` (ERROR.TYPE 11), but that is a guess.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) returns `ERROR.TYPE` and `ISERROR` of each error Excel stores as a rich value (`#SPILL!`, `#CALC!`, `#FIELD!`, `#BLOCKED!`, `#CONNECT!`, `#BUSY!`, `#UNKNOWN!`, uncached external ref) next to the cell itself; its cached oracle workbook gives each cell's errorType/subType.
- [ ] `rich_value_errors` maps every probed errorType; docs/excel-semantics/arrays.md lists the table with the probe as evidence.
- [ ] A `t="e"` cell whose `vm` cannot be resolved (rc out of range, no `errorType` key, missing field) also reads as `#UNDECODED!`, never the cached `#VALUE!`; unit test (T-0203 review).
- [ ] poi/link-external-workbook-b is refilled and scored (references.md entry updated).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine support for producing those errors.
