---
id: T-1576
title: "CalcResult and ArrayNode text carry Text (SPEC-0010 step 2)"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: sonnet
depends_on: [T-1575]
---

## Goal
Every evaluator text value (`CalcResult::String`, `ArrayNode::String`) is a `Text`, with no behavior change.

## Context
docs/specs/SPEC-0010-text-with-lone-surrogates.md "Where `Text` replaces `String`" and "Performance". Mechanical, compiler-driven migration of ~250 `CalcResult::String` and ~70 `ArrayNode::String` sites: producers move their `String` in (`.into()`), consumers read `&str` through `Deref` (the lossy view). Replace no helper's semantics yet; `utf16_slice`, `truncate_text` and `concat_text` keep working on the lossy view until T-1579/T-1580.

## Acceptance criteria
- [ ] `CalcResult::String(Text)` and `ArrayNode::String(Text)`; no new `clone`s on hot paths (a site that cloned a `String` clones the `Text`).
- [ ] Zero eval delta (`make eval` known failures unchanged).
- [ ] Outcome: `crates/rustsheet/benches/text.rs` (added by T-1575) before/after (release), within ±3%; aggregate as a sanity check.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lone-aware behavior in any function; `FormulaValue`/`SpillValue` (T-1577).

## Outcome
`CalcResult::String` and `ArrayNode::String` now hold a `Text`. Producers move their `String` in with `.into()`; consumers that need a `String` take the lossy view (`into_lossy_string()` / `to_string()`), so behavior is unchanged. `make check` is green and the eval has zero delta.

Bench (`cargo bench -p rustsheet --bench text`, release, fastest of 11 runs; origin/main → this branch, three paired runs on a noisy shared container):
- `A1&"x"`: 24.2/24.7/24.4 ms → 24.8/26.3/25.2 ms
- `MID(A1,2,3)`: 22.2/23.2/22.4 ms → 23.5/25.1/24.1 ms
- `LEFT(A1,2)`: 23.1/30.0/29.3 ms → 26.5/37.9/22.3 ms
- `CONCATENATE`: 73.2 → 74.8 ms
- `Text::push`: 54.7 → 54.7 ms

The run-to-run spread on the unchanged baseline (LEFT 23 to 30 ms) is larger than the ±3% budget, so the container cannot confirm the budget either way. The row-formula cases look 2 to 6% slower on average; T-1580 should re-measure on a quiet machine.
