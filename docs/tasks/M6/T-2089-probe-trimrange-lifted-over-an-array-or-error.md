---
id: T-2089
title: "Probe TRIMRANGE lifted over an array or error source"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's results for TRIMRANGE with a lifted trim_rows or trim_cols over sources T-1957 could not evidence are probed and the engine matches them.

## Context
T-1957 lifted TRIMRANGE into an array of references. Unprobed, implemented by analogy (UNVERIFIED in docs/excel-semantics/lookup.md): an array source (`INDEX(TRIMRANGE({1,2},{0;1}),1,1)`; the engine gives `#VALUE!` per element), a one-cell source that is not a written reference (defined name, `CHOOSE(1,A1)`, `INDIRECT("A1")`; the engine recovers a reference only for a written one), and an error source (`TRIMRANGE(NA(),{0;1})`; the engine gives one `#N/A`, static analysis predicts a 2x1). Use the `excel-probe` skill; code in functions/lookup_and_reference/trimrange.rs (`trimrange_reference`).

## Acceptance criteria
- [ ] A probe case holds the formulas above with Excel values; lookup.md records them as VERIFIED and the engine matches (baseline shrinks, no regression).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' lifts.
