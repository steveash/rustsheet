# 0148 — One GitHub client for the merge scripts: gh api behind scripts/gh_client.py

- **Status:** accepted (2026-10-28, T-3001). It implements SPEC-0027 section 4 ("GitHub access")
  and amends 0097 (where the PR-finished read lives).
- **Context:** SPEC-0027's merge-train scripts need GitHub reads and writes (PRs, comments with
  author permissions, labels, check and workflow runs, git commits, merge, workflow dispatch).
  `merge_lock.py` already had its own urllib read of a PR's state; a second and third client in
  `merge_ready.py` and `merge_train.py` would break CLAUDE.md directive 3. Reads through the
  session proxy were verified on 2026-10-28; writes were not.
- **Decision:** (being filled in by T-3001.)
- **Consequences:** (being filled in by T-3001.)
