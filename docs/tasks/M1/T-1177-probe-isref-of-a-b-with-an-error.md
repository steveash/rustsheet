---
id: T-1177
title: "Probe ISREF of a:b with an error or non-reference operand"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for `ISREF(a:b)` when an operand errors or is not a reference, with the engine matching it.

## Context
T-1166 made `is_reference` FALSE whenever `get_range` is not a range. Only the two-sheet form (probes/sheet-qualified-corner-order Calc!A7) has oracle evidence; see the UNVERIFIED note in docs/excel-semantics/references.md. Code: `is_reference` in crates/rustsheet-engine/src/functions/information/mod.rs.

## Acceptance criteria
- [x] Probe `ISREF(A1:INDEX(A1:A3,5))`, `ISREF(A1:(A1 B2))` and `ISREF(A1:IF(TRUE,1))` with the excel-probe skill; engine matches or the mismatch is fixed; the references.md entry is VERIFIED.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/isref-range-operands (27 cells, excel-graph 2026-10-09): the three acceptance cells were
already FALSE in the engine, as Excel has them. The probe also found that Excel treats an IF corner
of `:` as the reference it picks (`ISREF(A1:IF(TRUE,B2))` TRUE, `SUM(A1:IF(TRUE,B2))` 33), where the
engine gave `#VALUE!`. `range_operand` now resolves IF in reference context. references.md is
VERIFIED. `=A1:(1/0)` stays out of the probe because the Graph builder refuses it. Unit test
`range_operand_errors_values_and_if`. Eval: new probe case with all 27 cells passing (probes known
failures unchanged), no regressions.
