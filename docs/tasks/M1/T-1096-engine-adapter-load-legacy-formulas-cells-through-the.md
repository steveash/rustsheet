---
id: T-1096
title: "Engine adapter: load legacy_formulas cells through the legacy (implicit-intersection) parse"
milestone: M1
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The eval engine adapter honours a sheet's `legacy_formulas` list: those cells load through the legacy (implicit-intersection) parse that xlsx import uses, instead of as dynamic-array formulas.

## Context
T-1077 added `legacy_formulas` (spec2xlsx writes plain `<f>`) but the adapter ignores it (`crates/rustsheet-eval/src/engine.rs`, `case.rs`). So `probes/multiarea-at-legacy` H7-H10 (legacy `=A1:A2`, `=ABS(A1:A2)` and their FORMULATEXT: Excel gives `#VALUE!`, `#VALUE!`, `=@A1:A2`, `=ABS(@A1:A2)`) fail only because the engine evaluates them as dynamic-array, and H2/H4/H6 never exercise the engine's legacy printer. See T-0983/T-0985 for the import-side implicit intersection.

## Acceptance criteria
- [x] `multiarea-at-legacy` H7-H10 pass and their baseline entries are removed (`make eval-baseline`).
- [x] H2/H4/H6 still pass through the legacy path.
- [x] `make check` green; reviewers approve.

## Out of scope
- The SUM-of-union gap (H5), tracked by SPEC-0004.

## Outcome
The facade gained `Input::LegacyFormula` (backed by `Model::legacy_formula_text`, the same `add_implicit_intersection` xlsx import applies to a plain `<f>`), and the eval adapter enters a sheet's `legacy_formulas` cells through it. `multiarea-at-legacy` passes 10/10 (H7-H10 fixed, H2/H4/H6 pass through the legacy path) and `ifs-legacy-sumproduct` gains 17 cells. Eval: probes known failures 405 -> 384 (fixed 21), no regressions.
