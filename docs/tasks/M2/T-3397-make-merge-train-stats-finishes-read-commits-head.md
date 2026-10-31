---
id: T-3397
title: "make merge-train-stats finishes: read commits, head runs and jobs 4 at a time"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`make merge-train-stats` finishes in minutes on a day of real history, so the steward can record
T-2991's shadow evidence.

## Context
- Steward runs on 2026-10-30 reported that `make merge-train-stats` "hung" after 10 minutes. They
  blamed `gh` authentication, because `gh auth status` rejects the container's `GH_TOKEN`, and
  asked the owner to refresh that token.
- The token was never the problem. `GH_TOKEN` is a 14-character placeholder, and the session proxy
  authenticates the requests: `gh api` works, with an authenticated 15,000/h rate limit.
- The script was slow, not stuck. It read about 1,900 merge-lock push commits in a day, one
  `gh api` call each at about 0.3 s, because every waiter's refresh is a push. It also read every
  merged head's runs and every run's jobs one at a time.

## Acceptance criteria
- [x] `Stats.prefetch` fills the commit-message, head-run and job caches 4 reads at a time. The
      caches last one report run, so each object is read once per report. Tests show the
      read-once rule and that a failed read raises `GhError`
      (scripts/test_merge_train_stats.py `Prefetch`).
- [x] `make merge-train-stats` completes on main's last 24 h: 8 min 19 s with 4 readers (6 min with 8 or 16) on
      2026-10-31, with 135 merged PRs and 1,900 lock pushes. Before the change it printed nothing
      in 5 min here, and the steward abandoned it at 10 min.
- [x] scripts/test_merge_train_stats.py passes; `make hygiene` passes.

## Out of scope
- T-2991 itself (the switch to `train`).
- Removing `gh` from the scripts.

## Outcome
The 2026-10-31 07:48 UTC report gives T-2991's bar as "not met":
- 41 scored trains over 21 h, where 24 h is needed;
- agreement 87% (151/173), where 90% is needed.

Most disagreements are "predicted merge" on PRs that sat stranded with `ready-to-merge`, which is
the shadow-mode stranding T-3344 tracks: #2406, #2430, #2436 and #2474. Load is within the bar,
with PR CI p90 at 7.8 min against 6.5 min before (+20%) and job queue p90 at 3 s. Tree-check
mismatches: 0.
