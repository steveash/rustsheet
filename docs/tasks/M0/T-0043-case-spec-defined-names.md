---
id: T-0043
title: "Case spec: defined names"
milestone: M0
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
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
