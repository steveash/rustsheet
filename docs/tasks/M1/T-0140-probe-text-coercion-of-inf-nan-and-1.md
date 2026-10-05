---
id: T-0140
title: "Probe text coercion of inf, NaN and +1 in cast_number"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for whether text such as "inf", "NaN" and "+1" coerces to a number in arithmetic (`cast_number` currently uses Rust's `parse::<f64>`, which accepts them).

## Context
Found by the T-0131 fidelity review (existing gap, not introduced there). Use the `excel-probe` skill. Code: `Model::cast_number` in `crates/rustsheet-engine/src/cast.rs`; record the result in `docs/excel-semantics/coercion.md`.

## Acceptance criteria
- [ ] Oracle values for `="inf"*1`, `="NaN"*1`, `="+1"*1`, `="infinity"*1` recorded in a probe case.
- [ ] coercion.md entry; `cast_number` fixed if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
