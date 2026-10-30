---
id: T-3207
title: "Merge-train verdict records: record format, make verdict-record, mark derives verdicts, V4 checks them in warn mode"
milestone: M2
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2997]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
`make mark-ready` and the merger's V4 read per-reviewer verdict records bound to the head sha, so a stale or
review-less verdict is caught mechanically (in `warn` mode: reported, not yet enforced).

## Context
- Design: docs/specs/SPEC-0027-merge-train.md section 2, "Verdict records (v2)"; decision 0178; parent T-2997.
- Code: `scripts/merge_ready.py` (marker parse/render, `marker_comments`, `writer_markers`, `plan_mark`, `verify`
  V4, `Plan.actions`), `scripts/merge_step.py` (step schema: a record is a `comment` action),
  `scripts/merge_train.toml` and its reader (`make merge-mode`), `scripts/test_merge_ready.py`.
- Reuse the V1 writer filter (`Authors`) and the attribution-footer tolerance of decision 0175 for record bodies.

## Acceptance criteria
- [ ] A `merge-train-verdict v1` block (head, reviewer, verdict) with parse/render and tests, and a
      `make verdict-record PR=<n> REVIEWER=<name> VERDICT=<v> REPORT=<file> [STEP=1]` that posts the report plus
      the block for the local HEAD (refusing when HEAD is not the PR head), truncating to GitHub's limit.
- [ ] `mark` derives `reviewers:` from the newest writer record per applicable reviewer on the head; while
      `verdict_records = "warn"` (the default, read from `scripts/merge_train.toml` on `origin/main`) a missing
      record is a warning and `REVIEW=` still works; in `require` it is a refusal.
- [ ] V4 checks the records for `head` (or `prev` under a carry-over the merger re-verifies); in `warn` mode a
      gap is reported in the run summary and does not unlabel; in `require` it fails V4. Tests cover both modes,
      a stale-head record, a newer REQUEST_CHANGES and an outsider's record (ignored).
- [ ] `make check` green; security and architecture approve (protected paths: decision 0178 covers them).

## Out of scope
- The skills' instructions to post records and the flip to `require` (T-3208).
