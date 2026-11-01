---
id: T-2635
title: "Port ironcalc defined-name fixes (4ef43bed, 026f6c47, f8f68e27, 2aa0e653) and its Excel-authored named_ranges.xlsx"
milestone: M8
lane: engine
status: done
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
- [x] Each upstream case above is checked against our engine; the ones Excel's evidence (the
  workbook cells, or an oracle probe via the `excel-probe` skill) supports are unit tests that pass.
- [x] `named_ranges.xlsx` is in `crates/rustsheet-xlsx/tests/calc_tests/`, scored by `make eval`;
  new failures are fixed or baselined with owner tasks; no regressions.
- [x] Any new quirk is recorded in docs/excel-semantics/; commits carry `Ported-From: ironcalc@<sha>`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Upstream refactors that change no behavior (ff282eb1). If the gaps exceed size M, split them with the
  planner (lexer case first).

## Outcome
- 2aa0e653: `=Data!Q1_Total` is the workbook name `Q1_Total` (100), as Excel does
  (probes/defined-name-qualified-like-reference, excel-graph): the lexer reads a sheet-prefixed cell that more
  name characters (`_`, `.`, letters, digits; `Data!Q1.Rate` too, probes/defined-name-qualified-like-reference-more) follow as a qualified name (`consume_range`, lexer/mod.rs); unit test
  `qualified_name_spelled_like_a_cell_then_more`. Its FORMULATEXT cell (Excel shows the file name as prefix) is
  baselined and owned by T-2834.
- f8f68e27 is not Excel: names in a sheet-local name's body resolve as workbook names (`Data!DoubleRate` = `Rate*2`
  is 2, upstream 20; probes/defined-name-local-formula-scope, excel-graph, 10/10 pass already). Not ported.
- 4ef43bed, 026f6c47: upstream's `constants_and_expressions` and `sheet_qualified_names` cases already pass.
- `named_ranges.xlsx` (ironcalc ee4172e8) in calc_tests: 194/194 cells pass.
- Eval delta: +3 probe cases (21 of 22 cells pass, 1 new known failure owned by T-2834), +1 ironcalc-calc case
  (194 cells pass); no regressions. references.md has both findings.
