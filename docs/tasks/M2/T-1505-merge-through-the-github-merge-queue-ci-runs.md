---
id: T-1505
title: "Merge through the GitHub merge queue: CI runs on merge_group, agents enqueue with auto-merge"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
PRs merge through GitHub's merge queue, so a busy `main` no longer costs repeated update-and-retest
rounds, without losing decision 0028's guarantee that what lands was tested against current `main`.

## Context
Owner request (2026-10-13). Decision 0028's "branch must be up to date" rule plus a 13-minute CI
and about 2.7 merges an hour made PRs need several CI rounds (#586: five on 2026-10-12). The queue
setting is the owner's (repository admin); this task makes CI and the agents ready for it.

## Acceptance criteria
- [x] `ci.yml` runs on `merge_group`, with every job (the docs-only shortcut stays PR-only).
- [x] Decision 0053 records the rule and the exact owner settings; 0028 points to it.
- [x] The `steward` and `work-loop` skills and agent-workflow.md enqueue with `enable_pr_auto_merge`
      once the queue is on, and keep the 0028 procedure until then.
- [x] `make check` green; reviewers approve.

## Out of scope
- Turning the queue on (owner, repository settings).
- Speeding up the python job.

## Outcome
CI listens to `merge_group`; decision 0053 lists the owner settings (Require merge queue, squash,
groups of up to 5, the five required checks, "Allow auto-merge", up-to-date rule off). Skills switch
to enqueueing once the queue is on. The first PR after the switch verifies that
`enable_pr_auto_merge` enqueues (if it refuses a clean PR, the steward reports it and falls back).
