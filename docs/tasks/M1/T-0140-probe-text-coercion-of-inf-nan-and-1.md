---
id: T-0140
title: "Probe text coercion of inf, NaN and +1 in cast_number"
milestone: M1
lane: research
status: done
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
- [x] Oracle values for `="inf"*1`, `="NaN"*1`, `="+1"*1`, `="infinity"*1` recorded in a probe case.
- [x] coercion.md entry; `cast_number` fixed if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done inside T-0920 (the finite-number port needed the same evidence): probes/text-number-words
has oracle values for `="inf"*1`, `="NaN"*1`, `="+1"*1`, `="infinity"*1` and more ("inf"/"NaN"
are #VALUE!, "+1" is 1); `cast_number` and the other text-to-number sites were fixed;
coercion.md and numbers.md entries added.
