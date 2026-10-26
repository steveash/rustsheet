---
id: T-2784
title: "SPEC-0023 step 4: stored formula text keeps interior whitespace and drops leading whitespace on load"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2781]
---

## Goal
A formula loaded from a file's `<f>` text keeps its interior whitespace and drops a leading run, as Excel
does: `<f>1 + 2</f>` prints `=1 + 2`, `<f> 1+2</f>` prints `=1+2`.

## Context
SPEC-0023 step 4 ("Stored text"). Evidence: probes/t2776-formula-text-whitespace-shapes D1:E10 (excel-graph,
2026-10-26). Typed formulas keep the leading run (`= 1+2`), stored ones do not.

## Acceptance criteria
- [ ] t2776-formula-text-whitespace-shapes E1, E2, E6, E10 pass (E9 with T-2783); E7, E8 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- What Excel writes into `<f>` on save (T-2786).
