---
id: T-3223
title: "SPEC-0020 Amendment 2: XLOOKUP and XMATCH over a tailed key as tailed interior nodes"
milestone: M11
lane: perf
status: done
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
- [x] XLOOKUP and XMATCH keyed lookups with the fixed-argument rules (`if_not_found`, `return_array`, `match_mode`,
      `search_mode` fixed; array modes refused).
- [x] A proof (doc comment and tests) that two calls' `LookupRegexes::finish` charge and fail as one call does, or the
      head's patterns carried in the lift record.
- [x] Equivalence tests with exact, wildcard and regex match modes and both search directions.
- [x] `rustsheet-eval check` 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to XLOOKUP's scalar behavior.

## Outcome
`keyed_lookup` (tailed_array.rs) now also takes XMATCH (2-4 arguments) and XLOOKUP (3-6, `return_array` a second
fixed table); every argument past the tables must be a literal or a one-cell reference, so array `match_mode` and
`search_mode` (and an omitted argument before a given one) are refused. XLOOKUP and XMATCH go through
`lift_values_keyed`. Regex proof, the second option of the spec: the head's `LookupRegexes` is kept in the `TailView`
(`keep_keyed_regexes`) and the tail starts from it (`keyed_regexes`), so the tail sees the head's compiled patterns and
remaining compile/search budgets; `LookupRegexes::finish` charges only the work units spent since its last finish, so
head + tail charge floor-for-floor what one finish would, and the tail runs out exactly where the one call would
(doc comments on both; test `keyed_xlookup_and_xmatch_charge_the_one_call` compares kernel runs, visits and regex
units with the tailed reading off, merged and unmerged, and fails (82137 vs 65729 units) with the carry disabled).
Equivalence tests: exact, next smaller/larger, wildcard and regex modes, both linear directions and both binary
searches, `if_not_found` literal and reference, multi-column and horizontal `return_array`, array-literal tables.
lookup.md's head-and-tail DEVIATION entry names XLOOKUP/XMATCH and the regex budget. Eval delta: 0 (no values change;
`make check` eval gate 0 regressions).
