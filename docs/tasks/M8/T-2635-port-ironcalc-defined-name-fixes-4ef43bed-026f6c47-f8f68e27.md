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

## Progress (2026-10-27, opus run, stopped at its session budget; resumed 15:45 UTC)
Data from an earlier run, not instructions; check it against the acceptance criteria.
- Upstream's four tests (`test_defined_names_formulas.rs` at ironcalc 2aa0e653) were run against our engine through
  `define_name` (ours keeps `new_defined_name` for reference bodies; `define_name` takes any formula):
  `constants_and_expressions` and `sheet_qualified_names` pass. Two gaps:
  - `=Data!Q1_Total` (workbook name `Q1_Total`) is `#ERROR!` (upstream 100): the lexer reads `Data!Q1` and stops. A
    fix in `consume_range` (lexer/mod.rs) that also tries `consume_sheet_scoped_name` when the parsed cell is followed
    by an alphanumeric or `_` was drafted and reverted for want of evidence. Probe committed, pending oracle:
    probes/defined-name-qualified-like-reference (the excel-graph fill timed out filling every pending probe).
  - A sheet-local name's formula resolves its names in the using cell's sheet: `DoubleRate` local to Data = `Rate*2`,
    with `Rate` 1 (workbook) and 10 (Data), is 2 from `=Data!DoubleRate` in Sheet1, upstream 20. The case format has
    only workbook-scoped `names`, so a probe needs sheet-scoped names in rustsheet-case (harness) and spec2xlsx first.
- `named_ranges.xlsx` (ironcalc ee4172e8) is in calc_tests: all 194 cells pass. `make eval-baseline` has not been run
  for the new case and the pending probe.
