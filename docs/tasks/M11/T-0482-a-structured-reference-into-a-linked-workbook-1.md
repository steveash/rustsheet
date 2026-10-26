---
id: T-0482
title: "A structured reference into a linked workbook ([1]!Table3[#Data]) is #REF! (corpus-bench/58994 F8)"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
A formula that reaches a table in another workbook through an external-link index,
`VLOOKUP(E8,[1]!Table3[#Data],2,FALSE)`, evaluates to `#REF!`, as Excel does, instead of the engine's
parse-error `#ERROR!`.

## Context
Gap triage 2026-10-26 (external suites fetched). `corpus-bench/58994/1_58994_input.xlsx` is a whole-case known
failure with exactly one scored cell (`C corpus-bench/58994/1_58994_input.xlsx 1` and
`F corpus-bench/58994/1_58994_input.xlsx *` in eval/baseline/corpus-bench.txt) and no open task owns it:

- `Sheet1!F8` `=VLOOKUP(E8,[1]!Table3[#Data],2,FALSE)`: Excel `#REF!`, ours `#ERROR!`
  (`make eval-show CASE=corpus-bench/58994/1_58994_input.xlsx`; oracle: excel-graph sidecar in
  eval/suites/corpus-bench/oracle).

The workbook's `xl/externalLinks/externalLink1.xml` caches the other book's five `sheetNames` and empty
`sheetData` elements, and no `definedNames`, so nothing of `Table3` is cached. Excel still answers `#REF!`,
not the `#BLOCKED!` that an uncached *cell* or *sheet* reference gives (docs/excel-semantics/references.md
"External reference with no cached data is `#BLOCKED!`" and "Cached values of a linked workbook
(`[n]Sheet!A1`, `[n]!Name`, `[0]!Name`)"), which is the quirk worth recording: a structured (table) reference
qualified by a workbook is `#REF!` whatever the link caches.
Ours is `#ERROR!` because the lexer reads `[1]!Table3` as one name identifier (references.md, same entry) and
the following `[#Data]` then fails to parse: crates/rustsheet-engine/src/expressions/lexer/ and
`structured_reference` under crates/rustsheet-engine/src/expressions/parser/. The fix is to accept
`[n]!Name[...]` (and `[n]Sheet!Name[...]`) as a parse and give `#REF!` for it, not to resolve it.

## Acceptance criteria
- [ ] `Sheet1!F8` of corpus-bench/58994 is `#REF!`, so the `F corpus-bench/58994/1_58994_input.xlsx *` line
      disappears from eval/baseline/corpus-bench.txt, with no new F line in any baseline
      (`make eval`, `make eval-baseline`).
- [ ] An engine test: a formula with `[1]!Table3[#Data]` as a function argument is `#REF!`, and a plain
      same-workbook `Table3[#Data]` still resolves to the table's data area.
- [ ] docs/excel-semantics/references.md records the rule with this cell under the linked-workbook entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Resolving an external table from a link cache that does hold its cells (Excel gives `#REF!` here even so;
  file a follow-up if evidence ever shows otherwise).
- `[n]!NAME(...)` add-in calls, which are out of scope and unscored (T-0466).
- The other unowned corpus-bench cases (T-0480, T-0481).

## Visible-parity stage
Decision 0083: the one cell this task owns has the wrong error (not a last-digit difference), so the task is in
the campaign at the `semantics` stage. Its target is that the cell has Excel's `#REF!`; the first acceptance
criterion states it.
