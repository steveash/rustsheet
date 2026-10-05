---
id: T-0043
title: "Case spec: defined names"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A `rustsheet-case/v1` spec can declare workbook-scoped defined names, so behaviour that depends
on names gets oracle evidence in a committed case.

## Context
T-0138 probed a defined name whose formula is an intersection with a hand-built xlsx. The result
is recorded in docs/excel-semantics/references.md "A defined name may be an intersection", but it
cannot live in eval/suites because the case format has no names. That needs both sides:
`crates/rustsheet-eval/src/case.rs` (`CaseSpec`, loading into the engine) and
`tools/src/rustsheet_tools/spec2xlsx.py` (`define_name`).

## Acceptance criteria
- [x] Optional `"names": {"name": "=formula"}` in `CaseSpec`, loaded by the harness and written
  by spec2xlsx; tests on both sides.
- [x] A probes case for "A defined name may be an intersection" with an oracle sidecar, cited in references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Sheet-scoped names.

## Handoff (resumed)
Sonnet run, no code written. Findings:
- Harness side (`crates/rustsheet-eval/src/engine.rs::evaluate_spec`) builds the workbook only through the `rustsheet` facade (`Workbook::edit` / `Transaction`). The facade has no way to define a name, and SPEC-0001 (line ~25) lists "tables and names editing" as out of v0 scope. `Workbook.model` is `pub(crate)`.
- The engine can do it: `rustsheet_engine::Model::new_defined_name(name, scope, formula)` (model.rs ~3346).
- Decision needed: (a) add a narrow facade method (e.g. `Transaction::define_name(name, formula)`, workbook scope only) with a SPEC-0001 amendment/decision record, or (b) have the harness build its `Model` directly when a spec has `names` (a second load path; likely worse for "one definition per concept").
- Python side is straightforward: `build_xlsx` in `tools/src/rustsheet_tools/spec2xlsx.py` calls `wb.define_name(name, formula)` for each `spec.get("names", {})` entry (T-0138 probed it this way); add a test in tools/tests.
- Remaining: `names` field on `CaseSpec` (serde default), probes case for the intersection names (isect, wide, disj; see references.md "A defined name may be an intersection"), oracle fill with excel-graph, update that references.md entry to cite the case and move off UNVERIFIED.

## Outcome
- Chose option (a) from the handoff: a narrow facade op `Txn::define_name(name, formula)`. It is
  workbook-scoped, rolled back with the transaction, and errors with `EditError::DefinedName`.
  SPEC-0001 is amended for it. The engine gains `Model::define_name`, which accepts any body that
  parses, as Excel does; `new_defined_name` keeps its reference/LAMBDA restriction. The
  defined-name methods moved from `model.rs` to `defined_names.rs`, which paid the size ratchet
  down from 3829 to 3612 lines (decision 0010).
- `CaseSpec.names` is loaded by the harness and written by spec2xlsx, with tests on both sides.
- New case `probes/defined-name-intersection` with an excel-graph oracle sidecar; it matches the
  T-0138 scratch evidence and adds a constant name. references.md is now VERIFIED.
- Eval delta: probes known failures 61 → 70. All 9 are new coverage on the new case, with no
  regressions: the engine answers `#NAME?` for intersection and constant name bodies. Follow-up:
  T-0158 (engine).
