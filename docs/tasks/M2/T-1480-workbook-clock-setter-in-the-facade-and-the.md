---
id: T-1480
title: "Workbook clock setter in the facade and the Python binding"
milestone: M2
lane: api
status: done
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0008-workbook-clock.md
depends_on: [T-1479]
---

## Goal
Embedders set a workbook's clock through the public `rustsheet::Workbook` API and from Python
(`rustsheet.Workbook`), exactly as T-1478's spec defines.

## Context
T-1479 gives `Model` its own clock. This task exposes it: the facade method in crates/rustsheet/src/workbook.rs
(next to the feature-gated `pin_clock_from_metadata`, which the spec keeps or re-expresses), and the PyO3 method in
bindings/python. Public items need doc comments, and new public errors use `thiserror` (CLAUDE.md conventions).

## Acceptance criteria
- [x] The Rust facade method and the Python method match the spec's signatures, with doc comments.
- [x] Rust test: with a fixed clock, `=NOW()` and `=TODAY()` give the expected serials. After the clock changes, an `edit()`/`eval()` shows them updated as the spec says (ChangeSet).
- [x] A Python binding test does the same through `rustsheet.Workbook`.
- [x] SPEC-0001 (or the new spec) is marked implemented for this part; `make check` green; reviewers approve.

## Out of scope
- Engine internals (T-1479).

## Outcome
Added `Workbook::clock`, `Workbook::set_clock` and `Txn::set_clock` (undoable; a failed edit restores the clock) and the Python
`clock` keyword on `Workbook()`, `open` and `from_bytes` plus a `Workbook.clock` property whose setter is one edit
(`ValueError` out of range, `TypeError` for bool or non-int). Facade and Python tests cover the spec's test plan. Eval delta: none
(no engine change). SPEC-0008 and SPEC-0001 marked implemented.
