---
id: T-1594
title: "COUNTIF text criteria equality and unprobed characters use Excel collation"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTIF text criteria (the other *IF(S) functions share the code path; unprobed) `"x"`, `"=x"`, `"<>x"` compare with Excel's collation, and `cmp_text_excel` orders the characters T-1590 probed.

## Context
T-1590 probes: `probes/text-collation-lookups` (Calc!F1:H2, K1:K3, K8, K9 fail: `COUNTIF(r,"ae")` must count `æ`) and
`probes/text-collation-unprobed` (89 of 270 rank cells fail). Findings and the full observed order are in
docs/excel-semantics/coercion.md ("Collation equality in lookups and criteria") and criteria.md.
Code: `result_is_equal_to_string`, `result_is_not_equal_to_string` (functions/criteria.rs) use lowercased code points; route them through
`functions::util::collation::cmp_text_excel`. Extend `cmp_text_excel` for the letters/symbols in the doc (do not touch MATCH/VLOOKUP: they must not use collation equality, already passing).

## Acceptance criteria
- [ ] The failing cells of both probe cases pass; baseline shrinks (`make eval-baseline`).
- [ ] No regression in probes/text-collation-order.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Treating `œ`, `ñ`, `þ` etc. as expansions or separate letters (only their single-character positions are evidenced).
- Characters not in the T-1590 probes.
