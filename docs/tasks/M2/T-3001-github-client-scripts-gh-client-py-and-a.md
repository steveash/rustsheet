---
id: T-3001
title: "GitHub client scripts/gh_client.py and a write probe from a routine session"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-2985]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge-train scripts get one GitHub client. Before any slice depends on GitHub writes from a
session, we know whether those writes work through `gh api` in a routine session.

## Context
- SPEC-0027 section 4 ("GitHub access") and "Module placement": `scripts/gh_client.py` wraps
  `gh api`, and a fake implementation serves the tests.
- Operations the client needs:
  - PRs, and PR comments with each author's permission (`GET /collaborators/{user}/permission`);
  - labels;
  - check runs and workflow runs;
  - `GET /git/commits/<sha>`;
  - merge, with `expectedHeadSha` (the REST `sha` field);
  - workflow dispatch.
- Reads through the session proxy were verified on 2026-10-28. Pagination links that use numeric
  repository ids get HTTP 403 from the proxy, so the client pages with `repos/{owner}/{repo}`
  URLs and `page=`.
- `merge_lock.py`'s PR-finished read (decision 0097) moves onto this client, so there is one
  definition (CLAUDE.md directive 3).

## Acceptance criteria
- [x] `scripts/gh_client.py` and its fake implement the operations above.
      `scripts/test_gh_client.py` checks that both build the same request shapes, and runs in
      `make hygiene`.
- [x] `merge_lock.py` uses the client for the PR-finished read, and its tests still pass.
- [ ] Probe from a routine session (a worker run counts), with the evidence in the PR. It covers:
      - a comment, a label and an unlabel on a throwaway PR;
      - a push to `merge-train/probe`;
      - `GET /git/commits` on a PR's `merge_commit_sha`;
      - a squash merge of a throwaway PR with the right and with a wrong `expectedHeadSha`.

      The squash-merge probe uses a task-only PR that this task opens, merges under the merge
      lock and reverts if needed.
- [x] The answer (whether writes work, and which ones) is recorded in SPEC-0027 section 4 and in
      open question 1.
- [x] A decision record for the client lands in this PR. The architecture and security reviewers
      approve.

## Out of scope
- The step-mode fallback (T-2995, only if the probe fails).

## Outcome
- `scripts/gh_client.py`: `Client` builds every request of the operations above, with each URL
  value validated and lists paged with `page=` (at most 50 pages). `GhApi` sends requests with
  `gh api`, and `FakeGh` answers them from a table. `scripts/test_gh_client.py` pins both to the
  same recorded request shapes and the exact `gh` argv; it runs in `make hygiene`.
- `merge_lock.py`'s PR-finished read now goes through the client, with the same tests (decision
  0148).
- Probe, from this opus worker run on 2026-10-28 (SPEC-0027 section 4, "T-3001's probe"):
  - Reads work: PRs, comments, check runs, workflow runs, and `GET /git/commits` on #2160's test
    merge `fae51abf` and on its squash merge `13099cd6`.
  - The permission read `collaborators/{user}/permission` is refused by the proxy (HTTP 403).
  - Pushing an empty-tree commit to `merge-train/probe` (`62eebcac`) works.
  - **Not met:** the `gh api` comment, label and unlabel writes were refused by the session's
    tool-permission layer, so they never reached GitHub. This run did not try to get around
    that.
  - Squash merge with the MCP tool, on #2160 (this run's task-only PR), under the merge lock:
    the stale head `14419263` got HTTP 409 "Head branch was modified"; the current head
    `10f9c8aa` merged.
- Answer: sessions read with `gh_client` and write through the step mode (T-2995 builds it;
  T-2986 has its minimal fallback). T-2986's V1 and V6 need a session-safe author check, with
  `author_association` as the candidate. Owner question 1 now asks whether `gh api` writes should
  be allowed in routine sessions.
- Eval delta: none (scripts and docs only).

