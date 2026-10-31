# 0191 — merge-train-stats reads GitHub in parallel; gh needs no token of the owner's

- **Status:** accepted (2026-10-31, T-3397)
- **Context:**
  - Steward runs reported that `make merge-train-stats` hangs, and blamed `gh` authentication.
    The owner was asked to "refresh `GH_TOKEN`".
  - In the cloud containers, `GH_TOKEN` is a 14-character placeholder. `gh auth status` reports it
    invalid, but the session proxy authenticates every request, so `gh api` works with an
    authenticated rate limit (15,000/h, checked 2026-10-31).
  - The script was slow, not stuck. A day holds about 1,900 merge-lock pushes, because every
    waiter's refresh is a push. The script read their commits one `gh api` call at a time, at about
    0.3 s each, and did the same for each merged head's runs and each run's jobs.
- **Decision:**
  - `scripts/merge_train_stats.py` reads commit messages, head runs and run jobs 16 at a time
    (`READERS`), and caches each in memory. These objects never change once read.
  - Agents judge `gh` by whether `gh api` succeeds, never by `gh auth status`.
  - The owner supplies no GitHub token for the merge scripts.
- **Consequences:**
  - `make merge-train-stats` completes in about 6 minutes on a day of history, against more than
    10 before.
  - The steward can record T-2991's shadow evidence again.
  - The "refresh `GH_TOKEN`" owner item in docs/status.md is void.
  - 16 parallel reads stay far inside the rate limit.
