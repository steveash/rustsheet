---
id: T-1991
title: "Share one places-coercion helper across the nine base conversions"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The nine base conversions in functions/engineering/number_basis.rs coerce `places` through one shared helper instead of nine copies of the same block.

## Context
T-1816 moved the `places` coercion before the number coercion in all nine kernels (Excel reads the second argument first, probes/eng-first-vs-second-error). The block is repeated verbatim, so the ordering rule lives in nine places. A pure refactor; no behavior change.

## Acceptance criteria
- [ ] One helper owns the `places` coercion; eval output is unchanged (no baseline edits).
- [ ] Duplicated-line ratchet goes down (`scripts/check_duplication.py --update`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which error wins (T-1816).

## Handoff (resumed)
Refactor is done and approved by all four reviewers; `make check` passed locally and the eval is unchanged (PR #1384, branch agent/functions-engineering/T-1991-places-helper). CI hygiene fails in `scripts/check_governance.py`: `quality/duplication.json` (3176 -> 3147 via `scripts/check_duplication.py --update`) is a protected path and needs a decision record in the same PR. Next: either `make new-decision TITLE=...` for the ratchet bump and keep the json change, or restore the json to origin/main's value. Then rewrite the Outcome note (removed on handoff) and set `status: done`.
