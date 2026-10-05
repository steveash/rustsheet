---
id: T-0045
title: "Volatile analysis follows defined names (poi/56688_4.xlsx scores a TODAY-dependent cell)"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The harness never scores a cell whose value depends on a volatile function through a defined
name. It skips such a cell as volatile, as it already does for direct cell references (T-0005).

## Context
`poi/56688_4.xlsx` Sheet1!B2 is `=EDATE(DATE_CELL,INCREMENT)`. The workbook defines
`DATE_CELL` = `Sheet1!$A$2` and `INCREMENT` = `Sheet1!$A$3`. A2 is `=TODAY()`, cached on the day
Excel saved the file (41823), and A3 is 2. Excel cached 41885 in B2, the engine gives 44934 (the harness's fixed default clock, 2023-01-08) for
today's date, and the cell is a known failure (NumericMismatch) only because the harness scores
it. The workbook has no METADATA sheet, so the clock is not pinned and TODAY() itself is skipped.

Cause: `precedents` in `crates/rustsheet-eval/src/volatile.rs` only finds A1-style references
with a regex. Its doc comment says names are not followed. The xlsx snapshot
(`crates/rustsheet-eval/src/xlsx_snapshot.rs`, `WorkbookSnapshot`) does not carry defined names,
so the analysis cannot see them.

Suggested shape (keep it small):
- Read `<definedName>` entries from `xl/workbook.xml` into the snapshot: name, optional
  `localSheetId` scope, and body text. The JSON from `rustsheet-eval snapshot` gains a field. That
  is additive, but check that the `tools/` callers of `snapshot` ignore unknown fields.
- In the volatile analysis, a formula that mentions a defined name (as a whole word, outside string
  literals, not followed by `(`) also depends on the precedents of the name's body. A name body that
  is itself volatile taints the formula. Resolve sheet-scoped names before global ones. Guard
  against names that refer to each other in a cycle.
- Spec workbooks have no defined names yet (T-0043), so `volatile_cells` callers that have no names
  pass an empty map.

## Acceptance criteria
- [ ] `F	poi/56688_4.xlsx	Sheet1!B2` disappears from eval/baseline/poi.txt. The case's `C` line
      drops from 1 to 0 checked cells and the report counts the cell as skipped volatile. No new F
      lines in any suite, and no other case's checked count changes. If one does, the PR lists it
      with the reason.
- [ ] Unit tests in `volatile.rs`: a formula using a name that points at a TODAY() cell is
      tainted, a name whose body is `TODAY()` taints its users, a sheet-scoped name shadows a
      global one, and two names that refer to each other terminate.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluating defined names in the engine (T-0152 covers `Sheet1!#REF!` bodies). Following
  INDIRECT/OFFSET or 3-D references in the volatile analysis.
