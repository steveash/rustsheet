---
id: T-1701
title: "Probe a sheet rename under an undefined qualified name and a workbook name read through a sheet prefix"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Know, from Excel, what a sheet rename does to the prefix of (a) a qualified name that resolves to nothing (`=Data!Missing`) and (b) a workbook name read through a sheet prefix (`=Feb!loc`), and make the engine match.

## Context
T-1693 made a rename rewrite the prefix of a name scoped to the renamed sheet. The engine currently rewrites (a) (the parser gives it scope Some(Data) with an empty formula) but keeps the prefix of (b) (scope None); neither has oracle evidence. See `rename_defined_name_sheet` in `expressions/utils/mod.rs` and `prefixed_name_operand` in `parser/operands.rs`. Use the `excel-probe` skill; model the case on `probes/sheet-ops/sheet-ops-rename-sheet-scoped-name`.

## Acceptance criteria
- [x] Probe cases for (a) and (b) with oracle values, in `eval/suites/probes/cases/sheet-ops/`.
- [x] The engine matches them, or a follow-up task is filed; the finding is in `docs/excel-semantics/references.md`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Workbook-scoped names read without a prefix.

## Outcome
Probed on excel-graph (2026-10-26), findings in docs/excel-semantics/references.md:
- (a) `=Data!Missing` follows a rename of Data (FORMULATEXT `=Inputs!Missing`, `#NAME?`): ours matches
  (probes/sheet-ops/sheet-ops-rename-undefined-qualified-name).
- (b) Excel rewrites `=Feb!loc` (workbook-scoped `loc`) to `=<file>.xlsx!loc` on load, the file name quoted as a sheet
  name would be, so a rename never touches it; values stay 5 (probes/sheet-ops/sheet-ops-rename-workbook-name-via-prefix,
  and a control without the rename, sheet-ops-workbook-name-via-prefix-no-rename). Ours keeps `Feb!` in the text and
  gives `#NAME?` after the rename: follow-up T-2834 (engine, opus).
Eval: probes +3 cases (14 cells), 7 new-coverage known failures (owned by T-2834), no regressions.
