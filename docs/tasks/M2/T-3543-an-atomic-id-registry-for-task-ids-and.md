---
id: T-3543
title: "An atomic id registry for task ids and decision numbers"
milestone: M2
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Two sessions can never be handed the same task id or decision number, so the merge train stops
ejecting PRs for id collisions.

## Context
- Allocation scanned the tree and the pushed branches, so an id was safe only after its branch was
  pushed. On 2026-10-31 the train ejected or renumbered PRs for six collided ids. Decision 0215
  has the rule.

## Acceptance criteria
- [x] `scripts/id_registry.py` claims an id with one CAS push of the `id-registry` branch; a session
      that loses the race takes the next id (scripts/test_id_registry.py: claims in order, the race,
      refused picks, the newest-400 cap, the offline and unreachable fallbacks).
- [x] `make new-task` and `make new-decision` allocate through it.
- [x] The work-loop skill and task-format.md drop "push to claim" and forbid hand-picked ids.
- [x] The scripts tests pass.

## Out of scope
- A `make renumber` that renames a collided PR's ids mechanically.

## Outcome
Done in this PR. The test for two sessions claiming in the same second found that identical claim
commits let the losing push pass as "up to date"; each claim now carries a nonce.
