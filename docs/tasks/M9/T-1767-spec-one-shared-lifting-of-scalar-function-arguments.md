---
id: T-1767
title: "Spec: one shared lifting of scalar function arguments over arrays and ranges (stat, financial, engineering, date, lookup)"
milestone: M9
lane: engine
status: done
priority: P1
size: S
tier: opus
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
depends_on: []
---

## Goal
A spec (SPEC-0011) for one table-driven way to lift scalar-parameter functions over array and range arguments, so the statistical, financial, engineering, date and lookup families lift with one implementation instead of per-family copies.

## Context
Owner-directed work (2026-10-17): array lifting of scalar arguments is the largest remaining probe failure bucket. probes/stat-argument-modes (302 lift cells, T-1290), probes/fin-argument-modes (274, T-1372), probes/eng-argument-modes (about 94, T-1383), plus probes/date-argument-modes (T-1236), probes/lookup-argument-modes (T-1241) and probes/m7-argument-modes (T-1271) all fail the same way: `ROWS(F(..{v;v}..))` is 2 in Excel and `#N/IMPL!` or `#VALUE!` here. The legends are in docs/excel-semantics/statistics.md, financial.md, engineering.md, dates.md, lookup.md and arrays.md ("... function argument modes"). Lifting already exists in several shapes: `NumberOrArray` in cast.rs, `macros.rs`, date_and_time.rs `lift`, mathematical.rs, if_ifs.rs and the broadcast helpers (`bcast_get`, `bcast_idx`, T-1418, T-1122). Directive 3 (one definition per concept) and directive 4 (a spec before an architectural change) require one design before the per-family tasks.

## Acceptance criteria
- [x] docs/specs/SPEC-0011-scalar-argument-lifting.md: the Excel rules with evidence (which parameters lift over arrays, over ranges, or are taken whole; result shape and broadcasting of mismatched sizes, #N/A padding; error per element vs whole result; the ATP functions that refuse ranges), an inventory of today's lifting helpers, the shared design (per-function argument-kind table, one driver, bounded by check_array_size before any element is computed), migration of the existing helpers, and an ordered rollout of S/M tasks.
- [x] T-1290, T-1372, T-1383 (and T-1236, T-1241, T-1271 where they fit) get `depends_on` on the implementation task and a Context line pointing to the spec; new tasks filed for the rollout with scripts/tasks.py.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the families (T-1290, T-1372, T-1383 and the rollout tasks).

## Outcome
- docs/specs/SPEC-0011-scalar-argument-lifting.md, revised after the five-reviewer round on PR #987. The design:
  - A table in `functions/lift/kinds.rs` gives each parameter one of `Lift`, `LiftArrays` (Analysis ToolPak: arrays lift, a multi-cell range is `#VALUE!` with no intersection), `LiftArraysRangeWhole` (the D-functions' criteria), `Whole` or `WholeByCaller`, and each function a `KernelWork`.
  - The table wins: static analysis derives its signatures, static shapes and legacy `@` from it, and a test forbids old hand-written arms for table functions. The catalog's `lift: arrays` is derived from it.
  - The driver is in `functions/lift/driver.rs`. It is `lift_with_cost` moved (T-1784) and generalized (T-1770):
    - it reads every argument once, in a non-generic phase with an inline buffer, so an unlifted call has no heap and no grid machinery;
    - it bounds input cells, output cells (`check_array_size`) and kernel work (`LiftCost`, including `Prepared`) before any element;
    - it calls a borrowing scalar kernel per element (`&mut dyn FnMut`), in place when only one argument is a grid.
  - The spec inventories all existing lifting code: a migration task for each piece, or a non-goal with its reason. A final grep check is in T-1775.
- Probes (excel-graph, 2026-10-17): probes/scalar-lift-shapes, -b and -c (111 cells), recorded in docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule", with pointers in financial.md, engineering.md, dates.md and references.md. Each rule names the cells it rests on. The unprobed later ATP arguments, element values and precedence cases are listed and go to T-1777. The 90 failing cells are in eval/baseline/probes.txt. The counts in the spec are a snapshot (2026-10-17, ea76d91f).
- Directive-5 holes found:
  - `broadcast_text` has no array-size bound (T-1771, P0).
  - EXPAND, WRAPROWS, WRAPCOLS and MMULT allocate without `check_array_size` (T-1786, P0). HSTACK/VSTACK and TEXTSPLIT have the same hole (T-1780, T-1781, filed by the T-1771 work).
- Rollout:
  0. T-1771, T-1780, T-1781, T-1786.
  1. T-1784 (pure move), then T-1770 (table, hooks, EDATE, EOMONTH, WEEKNUM, YEARFRAC).
  2. In parallel: T-1236, T-1372, T-1290, T-1772 (the `Whole` + `Prepared` consumer), T-1383.
  3. T-1241, T-1271, T-1389.
  4. The old helpers: T-1773, T-1785, T-1774, T-1775 (last, with the grep check), T-1787.
  5. T-1776: derived catalog key, mapping, kernel-work and metamorphic checks.
  6. T-1777: open probes.
- Review: security, architecture, quality, excel-fidelity and performance all approved on the second round (PR #987); their minor notes are folded in.
