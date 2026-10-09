---
id: T-1287
title: "Probe PRODUCT with text, booleans and errors inside an array argument"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's PRODUCT of an array argument containing text, booleans and errors is established by oracle evidence and matches the engine.

## Context
T-1165 made PRODUCT multiply array arguments, skipping text/booleans and propagating errors by analogy with SUM and ranges; no probe covers it (docs/excel-semantics/coercion.md). Code: `crates/rustsheet-engine/src/functions/math_and_trigonometry/product.rs`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case holds Excel values for `PRODUCT({1,"a",TRUE,2})`, `PRODUCT({1,#N/A})`, `PRODUCT({"a"})`.
- [ ] Engine matches, or the difference is fixed; coercion.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
