---
id: T-3581
title: "range_values.rs module doc: say which pass caches read the range values, and which only use read_is_inert"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`range_values.rs`'s module doc names the pass caches that read range values through it, and says the SUM
running totals use only its inertness rule.

## Context
The security and quality reviews of T-3534 (PR 2659), both MINOR, found two doc slips. The module doc
said `sum_prefix.rs` reads through the range values, but it only calls `read_is_inert`. One doc line in
`ifs_cache.rs` ran past 100 columns. Both were deferred to the next engine change.

## Acceptance criteria
- [x] The module doc says which caches read the range values; the `ifs_values` doc line is re-wrapped.
      Comments only, no code change.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any code change.

## Outcome
Two doc comments corrected; no code changed.
