---
id: T-1709
title: "Probe LOOKUP with a boolean or empty scalar lookup_vector"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether LOOKUP reads a boolean or empty scalar lookup_vector as a one-element array, as it does a number or text one (T-1651), with ours fixed if it differs.

## Context
T-1651 (probes/mode-direct-error-lookup-scalar-vector B1-B5) showed LOOKUP reads a number or text scalar lookup_vector, written, `{1}` or IF-picked, as a one-element array, and dropped the IF/CHOOSE narrowing in `fn_lookup` (crates/rustsheet-engine/src/functions/lookup_and_reference/lookup.rs). `scalar_as_array` (cast.rs) also wraps Boolean and EmptyCell, which is unprobed: docs/excel-semantics/references.md lists it under "Inferred, unprobed". Raised by the T-1651 excel-fidelity review.

## Acceptance criteria
- [ ] A probe with oracle values for `LOOKUP(TRUE,TRUE)`, `LOOKUP(TRUE,TRUE,5)`, `LOOKUP(FALSE,TRUE)`, `LOOKUP(0,IF(TRUE,Z99))` with Z99 empty, `LOOKUP("",IF(TRUE,Z99),5)` and `LOOKUP(1,TRUE)`.
- [ ] references.md / lookup.md updated (the "Inferred, unprobed" note resolved); ours fixed if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
