---
id: T-0201
title: "Single criteria/wildcard matcher for *IF(S), D-functions and MATCH"
milestone: M2
lane: functions
status: done
priority: P2
size: M
depends_on: []
---

## Goal
Ensure SUMIF/COUNTIF/AVERAGEIF(S), D* and wildcard MATCH/XLOOKUP share one criteria parser and matcher with Excel semantics (operators, wildcards ~ escapes, case-insensitivity, numeric-text coercion, blanks).

## Context
Search for existing criteria code first; consolidate rather than add.

## Acceptance criteria
- [x] One module owns criteria; all consumers use it; spec cases for the grammar added to `smoke` (hand) and `probes` (oracle-pending).
- [x] No eval regressions; duplication not increased.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Criteria parsing/matching now lives in `functions/criteria.rs` (moved from `util.rs`).
`wildcard_text_matcher` replaces six copies of the wildcard-regex closure in MATCH, XMATCH,
XLOOKUP and the D-functions. Fixed `COUNTIF(range,"<>text")` ignoring non-text cells (Excel counts
numbers/blanks too). Spec cases: `smoke/criteria` (hand), `probes/criteria-grammar` (oracle-pending).
Follow-up: T-0205 (route D-function criteria through the shared parser).
Behavior note: D-function wildcard patterns that fail to compile now match nothing (previously fell through to prefix matching); unreachable in practice since patterns are escaped. Per-row matcher compilation in database.rs is left for T-0205.
