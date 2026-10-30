---
id: T-3301
title: "Port ironcalc 1e88d67b/5d6e7be2/fde5d637: defined names keep hidden and comment through xlsx import and export"
milestone: M8
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A defined name read from an xlsx keeps its `hidden` flag and its `comment` through import, the
API and export, so a saved workbook does not unhide names or drop their comments.

## Context
Upstream IronCalc 1e88d67b (hidden defined names), 819491ed (the flag in the Python bindings),
5d6e7be2 (defined-name comments), bbdfb385 (a saner getter) and fde5d637 (a struct for named
ranges) carry `<definedName hidden="1" comment="...">` from `xl/workbook.xml` through
`xlsx/src/import/workbook.rs`, the model and `xlsx/src/export/workbook.rs`, with the test
`xlsx/tests/hidden_defined_names.rs`. Our `types.rs` `DefinedName` has only `name`, `formula`
and `sheet_id`, so both attributes are lost on a round trip. Port the data path, not upstream's
API shape: our defined-name methods live in `defined_names.rs` and the facade/Python API is
SPEC-0001's (UPSTREAM.md "RustSheet-only engine additions"). Excel's own behavior for a hidden
name (it still evaluates; Name Manager hides it) needs no change to evaluation. Precedent:
T-2635. Triaged by the 2026-10-30 18:47 UTC steward run.

## Acceptance criteria
- [ ] `DefinedName` keeps `hidden` and `comment`; import reads and export writes both, with a round-trip test on a workbook that has a hidden, commented name.
- [ ] Evaluation of hidden names is unchanged; no eval regressions.
- [ ] The commit carries `Ported-From: ironcalc@fde5d637` (and the other shas it takes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New Python or facade API for setting the flags (file a follow-up in the api lane if wanted).
