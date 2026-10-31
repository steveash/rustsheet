---
id: T-3405
title: "Probe [#Headers] combined forms on a table without a header row"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3260]
---

## Goal
Excel evidence, and engine parity, for the structured-reference forms that combine `#Headers` with
a column or with `#Data` on a table that has no header row.

## Context
T-3260 (PR #2528) made `Sales[#Headers]` on a header-less table `#REF!`, as Excel does (oracle cell
F!A6 of probes/table-edits `table-no-header-baseline`). All four reviewers noted that two nearby
forms have no evidence:
- `Sales[[#Headers],[Column1]]` takes the same parser arm, so the engine now gives `#REF!`;
- `Sales[[#Headers],[#Data]]` (the `HeadersData` arm in
  `crates/rustsheet-engine/src/expressions/parser/operands.rs`) still resolves to the data rows.
Both are plausible and both are unprobed. `docs/excel-semantics/references.md` records the T-3260
results next to which the answer belongs.

## Acceptance criteria
- [ ] A new probes/table-edits case (a sibling of `table-no-header-baseline`, same `"header_row":
      false` table) asks Excel, through the oracle, for at least `ROWS`/`ROW`/`SUM`/`ISREF` over
      `Sales[[#Headers],[Column1]]`, `Sales[[#Headers],[#Data]]`, `Sales[[#Headers],[#Data],[Column2]]`
      and `Sales[[#All],[Column1]]`; the `.oracle.json` sidecar is committed.
- [ ] The engine matches every oracle cell (fix the parser arms if not), with a parser unit test next
      to the `#Totals` ones in `parser/tests/test_tables.rs`.
- [ ] The results are recorded in `docs/excel-semantics/references.md` beside the T-3260 entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Header-less tables imported from xlsx files Excel wrote (T-3260 left import unchanged).
