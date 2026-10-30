---
id: T-3208
title: "Merge-train verdict records: the review and work-loop skills post records, then flip verdict_records to require"
milestone: M2
lane: harness
status: todo
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
- [ ] The review skill posts each reviewer's report with `make verdict-record` as it returns (step mode when
      `gh api` writes are refused), and the work-loop and steward skills mark with `make mark-ready` deriving the
      verdicts (no `REVIEW=` once in `require`).
- [ ] After at least a week of `warn` mode in which every marked PR carried records (cite the merger's run
      summaries), a decision record flips `verdict_records` to `require` in `scripts/merge_train.toml`.
- [ ] `make check` green; security and architecture approve.

## Out of scope
- A second GitHub identity or an external signer (owner steps).
