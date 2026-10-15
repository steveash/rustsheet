---
id: T-1603
title: "Python test and values() for lone surrogates read back as str (after T-1579)"
milestone: M2
lane: api
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1579]
---

## Goal
A lone surrogate produced by a formula reads back from Python exactly, through `wb.value` and `wb.values`.

## Context
T-1578 made `Workbook.value` build the `str` from `text_utf16` (bindings/python/src/facade.rs) but could not test it from Python: no engine function produced a lone surrogate yet, and Python inputs cannot hold one. `values()` still returns the lossy text (U+FFFD); it needs the per-cell `text_utf16` lookup.

## Acceptance criteria
- [ ] Python test: `=MID("a😀b",3,1)` reads back as `'\ude00'` through `value` and `values`.
- [ ] A cell holding a genuine U+FFFD still reads back as U+FFFD (not routed through `text_utf16`).
- [ ] `values()` returns the exact `str` for lone-surrogate cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
