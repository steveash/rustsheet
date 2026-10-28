---
id: T-3001
title: "GitHub client scripts/gh_client.py and a write probe from a routine session"
milestone: M2
lane: harness
status: in-progress
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
- [ ] `scripts/gh_client.py` and its fake implement the operations above.
      `scripts/test_gh_client.py` checks that both build the same request shapes, and runs in
      `make hygiene`.
- [ ] `merge_lock.py` uses the client for the PR-finished read, and its tests still pass.
- [ ] Probe from a routine session (a worker run counts), with the evidence in the PR. It covers:
      - a comment, a label and an unlabel on a throwaway PR;
      - a push to `merge-train/probe`;
      - `GET /git/commits` on a PR's `merge_commit_sha`;
      - a squash merge of a throwaway PR with the right and with a wrong `expectedHeadSha`.

      The squash-merge probe uses a task-only PR that this task opens, merges under the merge
      lock and reverts if needed.
- [ ] The answer (whether writes work, and which ones) is recorded in SPEC-0027 section 4 and in
      open question 1.
- [ ] A decision record for the client lands in this PR. The architecture and security reviewers
      approve.

## Out of scope
- The step-mode fallback (T-2995, only if the probe fails).
