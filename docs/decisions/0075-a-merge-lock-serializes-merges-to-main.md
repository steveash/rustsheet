# 0075 — A merge lock serializes merges to main

- **Status:** accepted (2026-10-19, owner request, T-1937). It amends 0028 and replaces 0053's merge
  queue, which this repository cannot turn on.
- **Context:** Decision 0028 merges only a head that contains the current `main`, so a duplicate task
  or decision id (or any clash of two green PRs) fails in CI before it lands. Decision 0053 was to hand
  that guarantee to GitHub's merge queue, but the queue needs a repository owned by an organization
  and `steveash/rustsheet` belongs to a personal account (owner, 2026-10-19). Meanwhile `main` takes
  about 50 merges a day and CI takes about 5 minutes, so `main` often moves while a PR's CI runs: on
  2026-10-18, T-1861's PR #1115 merged `main` in five times before one green head was still current.
  Every round is a CI run and an agent wake-up, and with more agents it only gets worse.
- **Decision:** agents merge to `main` one at a time, under a lock.
  - The lock is the branch `merge-lock` on origin, written only by `scripts/merge_lock.py`
    (`make merge-lock-acquire PR=<n>`, `make merge-lock-release`, `make merge-lock-status`). It is an
    orphan commit with an empty tree whose message names the holder (`CLAUDE_CODE_SESSION_ID`) and the
    PR, or the free marker `state: free`. Every push carries `--force-with-lease`: the server creates
    the branch only while it is absent and replaces it only while it still holds the commit the caller
    read, so of two racing acquirers exactly one wins, and nobody can release someone else's lock.
    Release writes the free marker rather than deleting the branch: the session git proxy refuses
    branch deletion with HTTP 403 (found on the first live release, T-1938).
  - To merge a PR that is green and approved, an agent acquires the lock, merges `origin/main` into the
    PR head if it is behind, pushes, waits for CI on that head, runs 0028's up-to-date check and merges,
    then releases. With the lock held nobody else merges, so the check passes on the first green CI.
  - The holder releases at once when it stops for any reason: CI red, a conflict it cannot resolve
    quickly, review changes, the end of its session. It never holds the lock while it fixes code.
  - A lock older than 30 minutes is stale (its holder died) and the next acquirer takes it over, by the
    same compare-and-swap. A holder whose CI runs long renews it by acquiring again.
  - While another agent holds the lock, a waiting agent does other work (its next task) and tries
    again later, at its next PR event or check-in, or every few minutes while it has nothing else to do.
  - No agent merges a PR without holding the lock. The 0050 orphan-PR merges and the steward's own PR
    take the lock too. Task-only PRs as well: they are what collide on ids.
- **Consequences:** merges to `main` are serial, at most about one per CI run (about ten an hour), well
  above today's rate; a PR normally needs one CI run after it takes the lock, not one per `main` move.
  The guarantee rests on every merger following the steward skill, as 0028's check did; a human merging
  by hand bypasses it, which branch protection cannot stop on this plan either. If the repository moves to
  an organization and the merge queue is turned on, 0053 applies again and this lock is retired.
