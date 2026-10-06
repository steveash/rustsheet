---
id: T-0046
title: "Move is_excel_evidence next to the source-kind definition"
milestone: M0
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`runner.rs` no longer imports `is_excel_evidence` from `report.rs`; the predicate lives with the source-kind definition (case/suite) and both use it from there.

## Context
Architecture review of T-0044: scoring depends on the reporting module, a small layering inversion. Also note that `excel-cached (<app>)` case-level labels start with "excel" even for non-Excel apps, so the predicate must only be used on oracle `source.kind` strings, never on case-level labels.

## Acceptance criteria
- [ ] One definition of the predicate outside report.rs, with a doc comment on the label caveat; no behavior change.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
