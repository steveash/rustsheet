# 0145 — The merge lock serves waiters first-in, first-out

- **Status:** accepted (2026-10-28, T-2985). It amends 0075 and 0097; it implements SPEC-0027
  section 1 ("A fair merge lock").
- **Context:** Under 0075 a waiting agent retries later, so whoever retries first after a release
  wins, and the session that just released can win again at once. Over the 48 h before 2026-10-25,
  #1860 and #2000 (T-2755) each waited 30–40 min while one session chained #1859 and #1865, and
  the lock was taken over 37 times (SPEC-0027, Problem). Waiting by retrying also strands PRs when
  the session's budget ends mid-wait.
- **Decision:** the lock commit on `merge-lock` (format `version: 2`) holds both the holder and a
  queue of up to 20 waiters, so one compare-and-swap updates both, and the lock goes to waiters in
  the order they registered. The rules are SPEC-0027 section 1; in short:
  - `acquire` registers a caller that finds the lock busy (exit 1, with its place in the queue). A
    free, stale (30 min since `renewed`) or finished lock goes only to the head of the queue, or
    to anyone when the queue is empty. `release` keeps the queue, so a holder that releases and
    acquires again joins at the tail.
  - A waiter that has not refreshed for 10 min is dropped (refreshes are written at most every
    3 min); a head that does not claim a freed lock within 2 min, or whose PR is finished, is
    dropped. 0097's PR-finished read covers the head too; T-3010 rate-limits it to a re-read per
    PR every 10th poll.
  - A train lock (`pr: train`, `prs: ...`) is finished only when `prs` is non-empty and every PR
    in it is merged or closed.
  - Renewal and release retry when a waiter's refresh wins their compare-and-swap; a renewal that
    finds the lock taken over exits 4 (`lost`), distinct from 1 (`busy`). `acquire --renew` states
    that the caller holds the lock, so a takeover between two calls also exits 4.
  - Lock commits are validated on write and on parse: bounded ids and PR numbers, no newlines,
    timestamps clamped to the commit's committer time, and a missing or malformed holder reads
    as held by `unknown` (stale after the TTL), never free. Version-1 commits still parse.
  - With T-3010, agents wait with `make merge-lock-wait PR=<n>` in the background (polls every
    30 s, timeout 45 min by default) instead of retrying later; it exits 0 when the lock is theirs,
    3 when the PR is finished, 1 on the timeout. Until then a busy `acquire` keeps the caller's
    place for 10 min, so retrying within that time keeps it.
- **Consequences:** once T-3010 lands, this replaces 0075's "do other work and try again later":
  it changes the steward skill's "Merge lock" step 1 and work-loop step 6.3 to wait with
  `make merge-lock-wait`. The 30-minute TTL, the compare-and-swap and 0097's exit 3 are
  unchanged. During the rollout an older `merge_lock.py` ignores the queue and may take a lock
  reserved for the head; waiters then re-register at their next refresh. That loses fairness for
  a while, never mutual exclusion, and ends once every session starts from a `main` with T-2985. The PR-finished read moves to
  `scripts/gh_client.py` in T-3001.
