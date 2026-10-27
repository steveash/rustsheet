---
id: T-2910
title: "Audit M2's structural exit criterion: one aggregate-argument walker and one criteria matcher for every consumer, duplication down 15% from M2's start"
milestone: M2
lane: research
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The last M2 exit criterion that no tool measures has a recorded, checkable verdict. The criterion is "One
implementation of aggregate-argument iteration and criteria matching, used by every consumer. Duplication
baseline reduced by ≥ 15%" (docs/milestones.md, M2). The steward can then read M2's exit from the report's M2
row (T-2822) plus this record, without re-auditing the code.

## Context
- M2's cell criteria are close to met (`make eval` on 179909a8, the base of the planning branch): gen-m2 has 15,600 scored cells and 0 failing. The ironcalc-calc
  M2 function-scope row has 1 failing cell, 0 once exit exceptions are left out. The `M0-M2` probe row has 2
  visibly-off cells (probes/t2772-unary-plus-in-range-rows S!J2 and S!J5, owned by T-2885). T-2822 adds the M2
  exit row, but it keeps the structural criterion out ("not a cell measure"). Nothing records whether it holds.
- Aggregate iteration: SPEC-0009 (docs/specs/SPEC-0009-aggregate-argument-walker.md) put the walker in
  crates/rustsheet-engine/src/functions/util/aggregate_args.rs (T-1474, T-1476, T-1567, T-1677, all done). Today
  `AggregatePolicy` constants exist for SUM, MIN/MAX, PRODUCT, AVERAGE (three policies), the A-family, VARA,
  COUNT, COUNTA, AND/OR/XOR and the complex folds. `rg "let mut reference_error" crates/rustsheet-engine/src/functions`
  finds nothing outside the walker, which is SPEC-0009's check. Still open is whether every *consumer* in M2's
  sense is on it. Candidates to check include SUBTOTAL's own walk (functions/subtotal.rs `subtotal_walk_areas`),
  AGGREGATE (functions/aggregate.rs), LARGE/SMALL/PERCENTILE/RANK/MODE and SUMPRODUCT. For each one the audit
  decides one of three things: it is a consumer and is on the walker; its walk differs for an evidenced reason
  (SUBTOTAL skips hidden rows and nested SUBTOTALs); or it is a copy that needs a follow-up.
- Criteria matching: functions/criteria.rs (T-0201, T-0205) serves SUMIF/COUNTIF/*IFS (statistical/if_ifs.rs,
  ifs_cache.rs), the D-functions (database.rs, `CriteriaFlavor::Database`), and the wildcard matchers
  `WildcardPattern` and `wildcard_text_matcher` used by lookup_table.rs, xmatch.rs and text/position.rs (SEARCH).
  The audit lists every place that parses `<`, `>`, `=` operators or `~`, `*`, `?` wildcards and shows each goes
  through criteria.rs.
- Duplication: quality/duplication.json holds 2,931 (window 10) today. The status log records 9,153 → 8,829 at
  T-0200 (2026-10-02), the first M2 structural task. The window or the script may have changed since then, so
  re-measure both ends with today's scripts/check_duplication.py. The worktree clone is shallow: run
  `git fetch --unshallow` first, then measure on a worktree at the commit before T-0200 and at the PR's base.
  Which commit counts as "M2's start" is part of the verdict; justify it from git history.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) states the verdict on each half of the criterion. For aggregate
      iteration and for criteria matching it gives a table: function or call site, file, and whether it is
      on the shared code, differs for a cited evidenced reason, or is a copy.
- [ ] The record gives the duplication counts at the chosen M2 start commit and at the PR's base, both measured
      by today's scripts/check_duplication.py with the same window, the commits' hashes and the reduction in
      percent.
- [ ] Every copy found gets a follow-up task filed with `make new-task` (M2, engine or quality lane, naming
      the file and function and the shared code to move onto). Or the record explains why none is needed.
- [ ] docs/milestones.md's M2 structural bullet gains one line citing the decision record and its verdict, with
      no other change to the criteria text.
- [ ] No code changes in this PR. `make check` green; reviewers approve (architecture reviewer included).

## Out of scope
- Moving any remaining copy onto the shared code (the follow-up tasks do it).
- Adding the structural criterion to the report's M2 exit row (T-2822 keeps it out by design).
- Changing check_duplication.py or quality/duplication.json.
