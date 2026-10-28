# 0148 — One GitHub client for the merge scripts: gh api behind scripts/gh_client.py

- **Status:** accepted (2026-10-28, T-3001). It implements SPEC-0027 section 4 ("GitHub access")
  and amends 0097 (where the PR-finished read lives).
- **Context:** SPEC-0027's merge-train scripts need GitHub reads and writes (PRs, comments with
  author permissions, labels, check and workflow runs, git commits, merge, workflow dispatch).
  `merge_lock.py` already had its own urllib read of a PR's state; a second and third client in
  `merge_ready.py` and `merge_train.py` would break CLAUDE.md directive 3. Reads through the
  session proxy were verified on 2026-10-28; writes were not.
- **Decision:** `scripts/gh_client.py` is the only GitHub client of the merge scripts. `Client`
  builds every request (PRs, comments, the permission read, labels, check and workflow runs,
  `git/commits`, a merge pinned by `sha`, and workflow dispatch). It validates each value that
  goes into a URL and pages lists with `page=` (at most 50 pages). `GhApi` sends requests with
  `gh api` (REST), and `FakeGh` answers them from a table in tests. `merge_lock.py`'s
  PR-finished read (decision 0097) now goes through it: a failed read is still "open".
- **Consequences:**
  - `merge_ready.py` and `merge_train.py` import this client and never call GitHub any other way.
  - `merge_lock.py` needs `gh` on PATH for the PR-finished read; without `gh`, that read counts
    the PR as open, as an offline read did before.
  - T-3001's probe (SPEC-0027 section 4): reads and branch pushes work from a routine session.
    The proxy refuses the permission read (HTTP 403), and the session's permission layer refused
    the `gh api` comment and label writes. So sessions use the step mode (T-2995, and T-2986's
    minimal fallback), and V1 and V6 of T-2986 need a session-safe author check.
