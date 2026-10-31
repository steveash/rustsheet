# 0201 — Merge tooling deepens a shallow clone before its docs-only check

- **Status:** accepted (2026-10-31, T-3455). It amends nothing; it makes 0147's merge-base
  classification work in the clones that the merge tooling actually runs in.
- **Context:** The merge tooling's "green" rule (`merge_ready.ci_state`) accepts skipped heavy jobs
  only on a docs-only diff. It decides this with `ci_changes.classify_range`, which answers "code"
  in a shallow clone, because the merge base cannot be trusted there (0147). Cloud sessions clone
  the repo shallow (about 50 commits). In those clones, every task filing, gap triage and plan PR
  read as CI red and was ejected (#2574 at 13:40 UTC, #2577 at 13:54). No docs-only PR merged
  between the train being switched on (#2547, 10:20 UTC) and this record. Meanwhile the
  `functions` and critical lanes ran out of ready opus tasks while their filings sat ejected. A
  shallow session clone gives "code" for #2574 and #2577. After `git fetch --unshallow` (3 s,
  about 6 MB) the same clone gives "docs-only".
- **Decision:** `merge_ready.Repo.fetch_base` adds `--unshallow` to its fetch of `main` when the
  clone is shallow. Every docs-only check in the merge tooling (`verify`, `mark`, the train's
  `collect` and its tip check, merge-train-stats) runs after that fetch. `classify_range` itself
  is unchanged: in a shallow clone it still answers "code", and CI's own classifier still runs on
  a full checkout.
- **Consequences:** The merger's first `verify` in a fresh session costs one full-history fetch,
  done once per clone. Docs-only PRs that the bug ejected can be marked again (`make mark-ready`)
  and then merge through the train. A clone that cannot be deepened (fetch error) still fails
  `verify` loudly, as any failed read does. It never silently counts skipped jobs as green.
