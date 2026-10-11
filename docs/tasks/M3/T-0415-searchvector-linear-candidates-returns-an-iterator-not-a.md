---
id: T-0415
title: "SearchVector::linear_candidates returns an iterator, not a Vec"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A linear XMATCH/XLOOKUP search allocates no index vector: `SearchVector::linear_candidates` yields its candidate indices lazily.

## Context
Performance review of T-1264: `linear_candidates` (functions/lookup_and_reference/lookup_table.rs) collects a `Vec<usize>` as long as the evaluated prefix on every search, so a lifted XLOOKUP with k elements allocates k vectors of N indices besides its k*N comparisons. Return `impl Iterator<Item = usize>` (two chained ranges with the optional tail index) and update its callers (xmatch.rs `linear_search`, xlookup.rs `linear_search`). Behavior must not change.

## Acceptance criteria
- [ ] `linear_candidates` returns an iterator; no `Vec<usize>` is built per search; the XMATCH/XLOOKUP unit tests and eval cases are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
