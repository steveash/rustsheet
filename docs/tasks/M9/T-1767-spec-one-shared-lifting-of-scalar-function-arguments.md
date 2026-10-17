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
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the families (T-1290, T-1372, T-1383 and the rollout tasks).

## Outcome
- docs/specs/SPEC-0011-scalar-argument-lifting.md. The design:
  - `functions/arg_kinds.rs` gives each parameter one of `Lift` (arrays and ranges), `LiftArrays` (Analysis ToolPak: arrays only, a multi-cell range is `#VALUE!` with no intersection), `Whole` or `WholeByCaller`. Static analysis derives its `Signature`, the static shape of a call and the legacy `@` from that table.
  - `functions/util/lift.rs` holds the driver. It is `lift_with_cost` moved and generalized: it reads every argument once, broadcasts with `bcast_get` and `MISSING` (#N/A padding), and checks `check_array_size` and the cell-visit cap before any element. Then it calls a scalar kernel over values per element.
  - The spec inventories the nine existing lifting helpers and says how each one moves onto the driver.
- Probes (excel-graph, 2026-10-17): probes/scalar-lift-shapes, -b and -c (111 cells), recorded in docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule", with pointers in financial.md, engineering.md, dates.md and references.md. New findings:
  - ATP functions refuse a range in any argument, also on its own rows.
  - With an array next to the range, the call lifts over the array and every element is `#VALUE!`.
  - A legacy formula puts no `@` on an ATP argument's bare range (`=EDATE(A1:A60,1)` is `#VALUE!` on any row), but does put one on an operation on the range.
  - BITAND's later argument lifts over a range.
  - The shapes are padded with `#N/A` and the errors are per element, as for the operators.
  - The 90 failing cells are in eval/baseline/probes.txt.
- Found a directive-5 hole: `broadcast_text` has no `check_array_size` (`ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))` builds 2M cells). Filed as T-1771 (P0).
- Rollout:
  0. T-1771.
  1. T-1770: the driver, the table, static analysis, and EDATE, EOMONTH, WEEKNUM and YEARFRAC as first consumers.
  2. In parallel: T-1236 (rescoped to WORKDAY and NETWORKDAYS), T-1372, T-1290 (rescoped to the all-scalar stat functions), T-1772 (stat k/x arguments over a data array), T-1383.
  3. T-1241, T-1271, T-1389.
  4. The old helpers: T-1773, T-1774, T-1775.
  5. T-1776: catalog cross-check, `lift: arrays`, metamorphic test.
  6. T-1777: probe the open questions.
- Checks: `make hygiene`, `make typos`, `tasks.py validate`, `rustsheet-eval check` and `evidence-floor` pass. The reviewers have not run yet.

