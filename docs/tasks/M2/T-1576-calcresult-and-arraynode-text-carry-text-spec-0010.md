---
id: T-1576
title: "CalcResult and ArrayNode text carry Text (SPEC-0010 step 2)"
milestone: M2
lane: engine
status: in-progress
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
