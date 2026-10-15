---
id: T-1597
title: "Probe collation equality in SUMIF, SUMIFS and COUNTIFS criteria and in wildcard criteria against expansions (ae, ss)"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether the collation equality T-1590 found in COUNTIF's text criteria (`"ae"` counts `æ`, `"<>ae"` excludes it) holds in the rest of M2's *IF(S) family (SUMIF, SUMIFS, COUNTIFS) and in wildcard criteria, so T-1594's change to the shared criteria path is verified for every M2 consumer, not only COUNTIF.

## Context
M2 scope includes "criteria and wildcard grammar shared by the *IF(S) family" (docs/milestones.md). T-1590 (probes/text-collation-lookups, excel-graph 2026-10-15) evidenced collation equality for COUNTIF only; criteria.md "COUNTIF text criteria use the collation for equality (T-1590)" and coercion.md "Collation equality in lookups and criteria" record it. T-1594 (engine, todo) routes `result_is_equal_to_string`/`result_is_not_equal_to_string` (crates/rustsheet-engine/src/functions/criteria.rs) through `functions::util::collation::cmp_text_excel`; its Context says the other *IF(S) functions share that code path but are unprobed; of those, SUMIF, SUMIFS and COUNTIFS are M2 in eval/catalog/functions.yaml (AVERAGEIF(S), MAXIFS, MINIFS and the D-functions are M9). Also unprobed: wildcard criteria against expansions and folds, which go through `wildcard_text_matcher` (criteria.md "Wildcard text matching", verified case-insensitive with COUNTIF and plain ASCII only): does `"a?"` or `"a*"` match `æ`, does `"?"` match `æ` or `ß` (one character or two), does `"*ss*"` match `"straße"`, does `"æ*"` match `"aex"`. Use the excel-probe skill; tag the case `M2` (decision 0051, never M1) and call only M0–M2 functions in it, so every cell counts toward M2. Data: the T-1590 range {"æ","ae","a-e"} plus {"ß","ss","straße","aex","æx"} with numeric sums column.

## Acceptance criteria
- [ ] One new oracle-filled (excel-graph) probe case under eval/suites/probes tagged M2 with: `SUMIF`, `SUMIFS`, `COUNTIFS` with criteria `"ae"`, `"=ae"`, `"<>ae"`, `"æ"`, `"ss"`, `"<>ß"`; wildcard criteria `"a?"`, `"a*"`, `"?"`, `"*ss*"`, `"æ*"` in COUNTIF and SUMIFS.
- [ ] Findings recorded in docs/excel-semantics/criteria.md (extend the T-1590 entry: which functions it is VERIFIED for, plus a wildcard-and-expansions entry with Evidence/Status lines).
- [ ] Every mismatch with the engine on `main` (after T-1594 if it has merged; T-1594 is claimed on a pushed branch, so do not edit it) is filed as a new M2 engine follow-up task naming the failing cells; the case's failing cells go into eval/baseline via `make eval-baseline` and no other baseline line changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-1594 and any follow-ups do them).
- AVERAGEIF(S), MAXIFS, MINIFS and the D-functions (catalog milestone M9; probe them in an M9 case if needed).
- Characters not already ranked in probes/text-collation-order or probes/text-collation-unprobed.
