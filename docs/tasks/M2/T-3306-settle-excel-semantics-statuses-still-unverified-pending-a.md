---
id: T-3306
title: "Settle excel-semantics statuses still UNVERIFIED pending a probe task that is done (ROUND huge num_digits, D-function edge pool, CHISQ.DIST and BITAND limits, Bessel mismatches, IMLOG2, WORKDAY serial 0)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
No section of docs/excel-semantics/ says "UNVERIFIED (probe pending: T-xxxx)" or "UNVERIFIED until
T-xxxx ..." when T-xxxx is done. Each of the six sections below states its status from the evidence
that task left: VERIFIED with the case and cells, or still UNVERIFIED with the reason the probe did
not settle it and an open task that will.

## Context
docs/excel-semantics/ is the canonical quirk list (CLAUDE.md directive 6); a stale UNVERIFIED makes
agents re-probe settled rules or distrust them. Found by the planner on 2026-10-30 by matching every
UNVERIFIED line that names a pending task against the task statuses on main:
1. numbers.md "ROUND family with huge num_digits": "Status: UNVERIFIED (the rule separating the two
   outcomes needs a probe before implementing; T-0218)". T-0218 is done; its Outcome put the rule in
   numbers.md "ROUND family: huge num_digits wrap" (four probe cases, 578 cells). The old section
   duplicates that one (fold it in or replace it by a pointer; one definition per concept).
2. numbers.md "CHISQ.DIST degrees of freedom and BITAND operands are bounded": "UNVERIFIED until
   T-2421 commits the probe" (T-2421 done).
3. criteria.md "Database edge-pool smoke check (T-2249)": "Status: UNVERIFIED (probe pending: T-2422
   commits the probe case ...)" (T-2422 and T-2258 done).
4. engineering.md "Engineering edge-pool smoke check (T-2249)": "Status: UNVERIFIED (probe pending:
   T-2420 and T-2421 commit probe cases ...)" (both done).
5. engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)": "(UNVERIFIED
   until T-2044's sweep confirms them)" (T-2044 done).
6. dates.md "WEEKDAY of serial 0 and return_type 0": "UNVERIFIED for the WORKDAY/NETWORKDAYS values
   (probe pending, T-1254)" (T-1254 done).
For each: read the done task's Outcome, find the case it committed under eval/suites/ (`rg` the task
id), and check with `make eval-show CASE=<id>` that the cited cells pass or are known failures.

## Acceptance criteria
- [ ] Each of the six sections names its evidence (suite/case and cells) and says VERIFIED, or keeps
      UNVERIFIED with the reason and an open task id (`python3 scripts/tasks.py list` shows it open).
- [ ] numbers.md has one section for the ROUND huge num_digits rule, not two.
- [ ] `rg -n 'UNVERIFIED.*(pending|until)' docs/excel-semantics/` lists no line whose named tasks are
      all done (the planner's check above, rerun).
- [ ] Docs only: no change under crates/, eval/ or tools/. `make check` green; reviewers approve.

## Out of scope
- New probes. A section whose done task did not commit a probe gets a task, not a probe in this PR.
- Any other UNVERIFIED wording (most are open questions with open tasks).
