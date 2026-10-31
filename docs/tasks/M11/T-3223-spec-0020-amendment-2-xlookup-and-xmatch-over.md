---
id: T-3223
title: "SPEC-0020 Amendment 2: XLOOKUP and XMATCH over a tailed key as tailed interior nodes"
milestone: M11
lane: perf
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-3221]
---

## Goal
XLOOKUP and XMATCH over a tailed `lookup_value` are keyed lookups too, read as a head and a tail by every SPEC-0020
consumer with bit-identical results, lift charges and regex-cache outcomes.

## Context
Design: docs/specs/SPEC-0020-whole-column-array-operands.md "Amendment 2", section 3 "XLOOKUP and XMATCH" (Rollout step 3). They share the lift with VLOOKUP and MATCH
(`lift_values`, `REUSE_EQUAL_RUNS`), but each call owns a `LookupRegexes` cache that `finish` settles once per call.
Code: `functions/xlookup.rs` (`xlookup_impl`, `xlookup_if_not_found`), `functions/lookup_and_reference/xmatch.rs`,
`tailed_array.rs`.

## Acceptance criteria
- [ ] XLOOKUP and XMATCH keyed lookups with the fixed-argument rules (`if_not_found`, `return_array`, `match_mode`,
      `search_mode` fixed; array modes refused).
- [ ] A proof (doc comment and tests) that two calls' `LookupRegexes::finish` charge and fail as one call does, or the
      head's patterns carried in the lift record.
- [ ] Equivalence tests with exact, wildcard and regex match modes and both search directions.
- [ ] `rustsheet-eval check` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to XLOOKUP's scalar behavior.
