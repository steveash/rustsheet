# 0191 — merge-train-stats reads GitHub in parallel; gh needs no token of the owner's

- **Status:** accepted (2026-10-31, T-3397)
- **Context:**
  - Steward runs reported that `make merge-train-stats` hangs, and blamed `gh` authentication.
    The owner was asked to "refresh `GH_TOKEN`".
  - In the cloud containers, `GH_TOKEN` is a 14-character placeholder. `gh auth status` reports it
    invalid, but the session proxy authenticates every request, so `gh api` works with an
    authenticated primary rate limit (15,000/h, checked 2026-10-31). GitHub's secondary limits
    (about 900 REST points a minute, and a cap on concurrent requests) bound bursts.
  - The script was slow, not stuck. A day holds about 1,900 merge-lock pushes, because every
    waiter's refresh is a push. The script read their commits one `gh api` call at a time, at about
    0.3 s each, and did the same for each merged head's runs and each run's jobs.
- **Decision:**
  - `scripts/merge_train_stats.py` reads commit messages, head runs and run jobs 8 at a time
    (`READERS`). It caches each in memory for one report run, so each object is read once per
    report. A head's runs can change later, but one report's view of them does not need to.
  - Agents judge `gh` by whether `gh api` succeeds, never by `gh auth status`.
  - The owner supplies no GitHub token for the merge scripts.
- **Consequences:**
  - `make merge-train-stats` completes in minutes on a day of history (6 min with either 8 or 16 readers on
    2026-10-31); before, it ran past 10 minutes and was abandoned.
  - The steward can record T-2991's shadow evidence again.
  - The "refresh `GH_TOKEN`" owner item in docs/status.md is void. The steward removes it at
    its next run.
  - Eight readers burst at about 25 requests/s, under the 900-a-minute secondary limit, and the
    total request count does not grow. A run that still hits a secondary limit gets a 403 and fails
    loudly (`GhError`, exit 2), never with wrong numbers.
