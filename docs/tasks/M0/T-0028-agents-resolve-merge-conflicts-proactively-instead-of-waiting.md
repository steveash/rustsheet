---
id: T-0028
title: "Agents resolve merge conflicts proactively instead of waiting on CI"
milestone: M0
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
No agent PR sits conflicted while its agent waits for CI that GitHub will never run.

## Context
2026-10-04: #56 (T-0019) and #53 (T-0206) both went `dirty` when #54 merged; GitHub ran no CI on them, so the
session waiting for a CI event was stuck until the owner asked. Conflicts were mechanical (status log, BOARD.md,
baselines). Owner asked for the instructions to make conflict resolution proactive.

## Acceptance criteria
- [x] `steward` skill: check `mergeable_state` after every push, every event/check-in and every merge to main;
      `dirty` → merge main, resolve, `make check`, push; arm a safety-net check-in after opening a PR.
- [x] work-loop step 3 points to it; decision 0020 records it.
- [x] `make check` runs `typos` (CI-only before; #56 and #53 went red on it after a green local check).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Routine prompt text (docs/process/routines.md): the routines read these skills, so no re-apply is needed.

## Outcome
Steward skill and work-loop step 3 updated (decision 0020). Applied the same day to #56 and #53.
