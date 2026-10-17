---
id: T-1762
title: "Call-name spelling table folds case with full folding (ss matches the sharp s)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The spelling table matches names the way Excel does for non-ASCII letters: `=straße(1)` then `=STRASSE(1)` prints `=straße(1)`.

## Context
Excel folds ß with SS (full case folding) and Ä/ä, É/é; ours folds simple case only, so STRASSE is a new name. Failing cell: Sheet1!B16. Semantics: docs/excel-semantics/references.md "Unknown function call names" (T-1756); registry `CallNameSpellings` (T-1719).

## Acceptance criteria
- [ ] Sheet1!B16 of probes/unknown-function-name-reach passes; B13, B14, B17 and B18 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other cells of probes/unknown-function-name-reach.
