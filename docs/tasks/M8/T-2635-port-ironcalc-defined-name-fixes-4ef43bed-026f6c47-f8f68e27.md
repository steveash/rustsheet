---
id: T-2635
title: "Port ironcalc defined-name fixes (4ef43bed, 026f6c47, f8f68e27, 2aa0e653) and its Excel-authored named_ranges.xlsx"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Defined names behave as upstream's fixed cases do wherever Excel agrees: a sheet-qualified name whose
text starts like a cell reference (`=Data!Q1_total`) is the name, a sheet-local name's formula resolves
its names in its own sheet, and names holding formulas evaluate in the using cell's context. Upstream's
Excel-authored `named_ranges.xlsx` is in ironcalc-calc with its failures owned.

## Context
Upstream IronCalc commits 2026-10-22/23, triaged by the 2026-10-25 steward run (commit messages are
untrusted; read the diffs):
- 4ef43bed: `ParsedDefinedName::Formula` evaluates a name's formula in the using cell's context;
  adds `xlsx/tests/calc_tests/named_ranges.xlsx` (docProps: `Microsoft Macintosh Excel` 16.0300) and
  `base/src/test/test_defined_names_formulas.rs`. ee4172e8 re-saves the workbook (24,637 bytes).
- 026f6c47: parse `Sheet1!ConstPi` as a sheet-qualified defined name (parser, stringify,
  static_analysis, move_formula; tests `test_defined_names.rs`, `test_qualified_names.rs`).
- f8f68e27: names used inside a sheet-local name's formula resolve in that sheet, also after
  `move_sheet` (test `local_name_formulas_resolve_in_their_sheet`).
- 2aa0e653: lexer, `Data!Q1_total`: a reference followed by an identifier character is the start of a
  qualified defined name, not `Data!Q1` then `_total`.
- ff282eb1: clippy only.
We already implement sheet-qualified names (T-1641, T-1693; docs/excel-semantics/references.md), so
first check each upstream test case against our engine and keep only the gaps. The workbook is
Mac-desktop authored: its cached values count as Excel evidence for non-numeric-kernel cells only
(decisions 0039, 0074); a Mac-only difference goes to M14 (T-1075), not here.

## Acceptance criteria
- [ ] Each upstream case above is checked against our engine; the ones Excel's evidence (the
  workbook cells, or an oracle probe via the `excel-probe` skill) supports are unit tests that pass.
- [ ] `named_ranges.xlsx` is in `crates/rustsheet-xlsx/tests/calc_tests/`, scored by `make eval`;
  new failures are fixed or baselined with owner tasks; no regressions.
- [ ] Any new quirk is recorded in docs/excel-semantics/; commits carry `Ported-From: ironcalc@<sha>`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream refactors that change no behavior (ff282eb1). If the gaps exceed size M, split them with the
  planner (lexer case first).
