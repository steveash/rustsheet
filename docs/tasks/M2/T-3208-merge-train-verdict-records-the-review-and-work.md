---
id: T-3208
title: "Merge-train verdict records: the review, work-loop and steward skills post and use records"
milestone: M2
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: [T-3207]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
Every marked PR carries a verdict record per reviewer, and after a clean week in `warn` mode V4 enforces them.

## Context
- Design: docs/specs/SPEC-0027-merge-train.md section 2, "Verdict records (v2)"; decision 0178; parent T-2997.
- Needs T-3207 (`make verdict-record`, the `warn`/`require` key).
- The review skill (`.claude/skills/review/SKILL.md`) step 4 consolidates the reviewers' reports; the work-loop
  (step 6.3) and steward ("Merge train") skills describe marking.

## Acceptance criteria
- [x] Each reviewer agent's report starts with the `git rev-parse HEAD` it reviewed (SPEC-0027 "Verdict
      records (v2)").
- [x] The review skill posts each reviewer's report with `make verdict-record` as it returns (step mode when
      `gh api` writes are refused), and the work-loop and steward skills mark with `make mark-ready` deriving the
      verdicts (no `REVIEW=` once in `require`).
- [x] `make check` green; security and architecture approve.

## Out of scope
- The flip to `require` after a week of `warn` data: split into T-3243 (it cannot finish within one session).
- A second GitHub identity or an external signer (owner steps).

## Outcome
Done 2026-10-30. The five reviewer agents' output starts with a `HEAD:` line (the commit reviewed). The `review`
skill posts each reviewer's report as a verdict record with `make verdict-record` (step mode when `gh api`
writes are refused; `BLOCK` recorded as `REQUEST_CHANGES`; nothing posted for a moved HEAD); the work-loop opens
the PR as a draft before the first review round so the records have a PR; the work-loop and steward skills mark
with `make mark-ready PR=<n>`, `REVIEW=` only standing in for a missing record in `warn`. The flip to `require`
is T-3243 (blocked until 2026-11-07 for a week of `warn` data; it also carries the security MINOR from PR #2419).
Decision 0178 records both. Eval delta: none (skills and docs only).
