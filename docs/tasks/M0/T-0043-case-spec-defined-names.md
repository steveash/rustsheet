---
id: T-0043
title: "Case spec: defined names"
milestone: M0
lane: harness
status: todo
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
- [ ] Optional `"names": {"name": "=formula"}` in `CaseSpec`, loaded by the harness and written
  by spec2xlsx; tests on both sides.
- [ ] A probes case for "A defined name may be an intersection" with an oracle sidecar, cited in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheet-scoped names.

## Handoff
Sonnet run, no code written. Findings:
- Harness side (`crates/rustsheet-eval/src/engine.rs::evaluate_spec`) builds the workbook only through the `rustsheet` facade (`Workbook::edit` / `Transaction`). The facade has no way to define a name, and SPEC-0001 (line ~25) lists "tables and names editing" as out of v0 scope. `Workbook.model` is `pub(crate)`.
- The engine can do it: `rustsheet_engine::Model::new_defined_name(name, scope, formula)` (model.rs ~3346).
- Decision needed: (a) add a narrow facade method (e.g. `Transaction::define_name(name, formula)`, workbook scope only) with a SPEC-0001 amendment/decision record, or (b) have the harness build its `Model` directly when a spec has `names` (a second load path; likely worse for "one definition per concept").
- Python side is straightforward: `build_xlsx` in `tools/src/rustsheet_tools/spec2xlsx.py` calls `wb.define_name(name, formula)` for each `spec.get("names", {})` entry (T-0138 probed it this way); add a test in tools/tests.
- Remaining: `names` field on `CaseSpec` (serde default), probes case for the intersection names (isect, wide, disj; see references.md "A defined name may be an intersection"), oracle fill with excel-graph, update that references.md entry to cite the case and move off UNVERIFIED.
