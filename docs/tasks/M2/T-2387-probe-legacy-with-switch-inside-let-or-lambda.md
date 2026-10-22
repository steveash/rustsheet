---
id: T-2387
title: "Probe legacy @ with SWITCH inside LET or LAMBDA, a defined name, and other dynamic-array-only functions"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's legacy `@` behavior is known for a SWITCH inside a LET or LAMBDA body, a SWITCH reached through a defined name, and for other dynamic-array-only functions (XLOOKUP, FILTER, LET), and ours matches.

## Context
T-2121 found that a SWITCH anywhere in a legacy formula removes every automatic `@` (`add_root_legacy_intersection`, `Node::contains_function`, so it also fires for a SWITCH inside a LET/LAMBDA body). Those shapes and other new functions are unprobed; see docs/excel-semantics/references.md "Generalised (probes/legacy-switch-callers ...)". Use the excel-probe skill with legacy cells and FORMULATEXT twins like legacy-switch-callers.

## Acceptance criteria
- [ ] A probes case covers the shapes with oracle values; the parser matches them or the findings are recorded in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
