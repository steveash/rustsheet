---
id: T-3243
title: "Merge-train verdict records: flip verdict_records to require after a clean week of warn"
milestone: M2
lane: harness
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-3208]
spec: docs/specs/SPEC-0027-merge-train.md
---

Blocked because: waits for a week of `warn` data; unblock (status `todo`) on or after 2026-11-07, once
T-3208 has been on `main` for seven days.

## Goal
V4 enforces the verdict records: `verdict_records = "require"` in `scripts/merge_train.toml`.

## Context
- Split from T-3208 (decision 0178's rollout): T-3207 built the records and `warn` mode, T-3208 made the review,
  work-loop and steward skills post them. The flip waits for evidence that every marked PR carries records.
- Evidence: `make merge-ready-verify` and the merger's run log (dry-run and shadow/train runs) print
  `verdict records #<n> (warn): ...` for each gap; a PR marked with `REVIEW=` standing in prints
  `merge_ready: warning: ...` at marking. Read the marker and record comments of the PRs marked in the week.
- Security MINOR from PR #2419: `Repo.switch_text` reads any `git show` failure as an absent switch file, which
  is `warn`. With the flip, make an absent file or key read as `require` too (only a fetched `main` whose switch
  file says `warn` is `warn`), so a lost file cannot silently relax V4.

## Acceptance criteria
- [ ] A decision record cites the week's evidence: every PR marked since T-3208 merged had a record per
      applicable reviewer on its marked head (or names the exceptions and why they are harmless).
- [ ] `verdict_records = "require"` in `scripts/merge_train.toml`; `records_mode` reads a missing file or key
      as `require`; tests updated.
- [ ] The work-loop and steward skills drop the `REVIEW=` stand-in wording.
- [ ] `make check` green; security and architecture approve.

## Out of scope
- A second GitHub identity or an external signer (owner steps).
