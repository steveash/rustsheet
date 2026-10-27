---
id: T-2785
title: "Accept whitespace after a sheet prefix's bang (=S! B1 reads S!B1)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=S! B1` (whitespace after a sheet prefix's `!`) reads `S!B1` as Excel does, instead of a parse error, and
prints as `=S!B1`.

## Context
Found by T-2776: probes/t2776-ws-after-bang A1 (Excel 1, ours `#ERROR!`) and C1 (Excel prints `=S!B1`).
docs/excel-semantics/syntax.md "Formula text keeps typed whitespace…" (Dropped: after a sheet prefix's `!`).
Whitespace around `:` is already accepted and dropped (t2776-ws-before-colon / -after-colon pass).

## Acceptance criteria
- [x] probes/t2776-ws-after-bang A1, C1 pass; 0 regressions.
- [x] Quoted sheet prefixes (`='My S'! B1`) behave the same, or are probed first if unsure.
- [x] `make check` green; reviewers approve.

## Out of scope
- Whitespace inside the sheet name or before `!` (probe first if needed).

## Outcome
`consume_range` (lexer) now skips whitespace after a sheet prefix's `!`, for plain and quoted prefixes
alike (the quoted case shares the path; unit test in `lexer/test/test_ranges.rs`, not separately oracle-probed).
probes/t2776-ws-after-bang A1 and C1 now pass (2 cells fixed, 0 regressions; baseline updated).
