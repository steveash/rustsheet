---
id: T-2297
title: "Move and grow tables on structural edits (SPEC-0016 step 1b)"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2195]
---

## Goal
A row or column edit wholly before a table moves the table, and a row insert or delete strictly inside its
data rows grows or shrinks it, so the structured references to it keep their text through the edit.

## Context
Split from T-2195 (its split hint, part b). T-2195 keeps every structured reference as written but treats
every table as SPEC-0016's *fallback*: an edit that changes the area a reference resolves to rewrites it as
the displaced plain range (`StructuredReferenceKind` arm of `stringify`, parser/stringify.rs), and tables
never move. This task adds the *move* and *grow/shrink* classes of SPEC-0016 "Structural edits": classify
the tables on the edited sheet before the walk by the spec's edge-case table, pass the fallback table names
to `to_string_displaced` (a reference to a moved or grown table prints its text), update the tables'
`reference` after the walk, then `Model::reset_tables`. The interim rule is a hand assumption
(docs/excel-semantics/references.md "Structured references when the table changes"); T-2197 probes Excel.
Tests: `src/test/test_structured_references.rs` (`check_edit` pins today's values with plain-range twins).

## Acceptance criteria
- [ ] Each row of SPEC-0016's edge-case table and each column case has a test: move and grow/shrink keep
  the structured text, fallback prints today's lowered text; values equal today's in every case.
- [ ] One test has the formula on another sheet; defined names and CF formulas follow the same rule.
- [ ] `make eval` shows no regression; `make check` green; reviewers approve.

## Out of scope
-
