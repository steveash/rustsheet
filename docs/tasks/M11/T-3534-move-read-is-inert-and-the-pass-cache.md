---
id: T-3534
title: "Move read_is_inert and the pass-cache read rule out of ifs_cache.rs to a neutral home shared by the criteria and SUM prefix caches"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The rule that lets a pass cache skip a cell read (`read_is_inert`, T-0449) lives in one neutral place
beside the pass-cache plumbing of `Evaluation`, and every pass cache (criteria, SUMPRODUCT blocks, SUM
prefixes) uses it from there.

## Context
T-3456 (`functions/math_and_trigonometry/sum_prefix.rs`) widened `read_is_inert` in
`functions/statistical/ifs_cache.rs` to `pub(crate)`, so a math function now depends on a statistical cache
file (architecture review of T-3456, MINOR). `pass_cache_safe` already lives in `evaluation.rs`.

## Acceptance criteria
- [x] `read_is_inert` is defined once outside `functions/statistical/`, and `ifs_cache.rs` and
      `sum_prefix.rs` call it there; behavior unchanged (existing cache tests pass unmodified).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing what counts as inert.

## Outcome
`read_is_inert` (from functions/statistical/ifs_cache.rs) and `pass_cache_safe` (from evaluation.rs) now live
together in a new crate-level module, `crates/rustsheet-engine/src/pass_cache.rs`: the two rules every pass cache
shares. `ifs_cache.rs` and `sum_prefix.rs` call them as `Model` methods, as before. The bodies are moved verbatim, so
behavior is unchanged and no test changed. evaluation.rs (982 lines) could not take `read_is_inert` without
breaking the 1000-line file limit, hence the new module. Eval delta: none (move only).
