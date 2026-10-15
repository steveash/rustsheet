---
id: T-1594
title: "COUNTIF text criteria equality uses Excel collation"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTIF text criteria `"x"`, `"=x"`, `"<>x"` compare with Excel's collation (the other *IF(S) functions share the code path; unprobed).

## Context
T-1590 probes: `probes/text-collation-lookups` (Calc!F1:H2, K1:K3, K8, K9 fail: `COUNTIF(r,"ae")` must count `æ`) and
(the rank cells of `probes/text-collation-unprobed` moved to T-1595). Findings and the full observed order are in
docs/excel-semantics/coercion.md ("Collation equality in lookups and criteria") and criteria.md.
Code: `result_is_equal_to_string`, `result_is_not_equal_to_string` (functions/criteria.rs) use lowercased code points; route them through
`functions::util::collation::cmp_text_excel`. The character order is T-1595 (do not touch MATCH/VLOOKUP: they must not use collation equality, already passing).

## Acceptance criteria
- [x] The failing cells of probes/text-collation-lookups pass; baseline shrinks (`make eval-baseline`).
- [x] No regression in probes/text-collation-order.
- [x] `make check` green; reviewers approve.

## Out of scope
- Treating `œ`, `ñ`, `þ` etc. as expansions or separate letters (only their single-character positions are evidenced).
- Characters not in the T-1590 probes.

## Outcome
`result_is_equal_to_string` and `result_is_not_equal_to_string` compare with `cmp_text_excel`. 11 cells of probes/text-collation-lookups fixed (baseline 5596 -> 5585), regressions 0. The character-order half is T-1595 (opus). Unit test: `test_text_equality_criteria_use_collation`.
