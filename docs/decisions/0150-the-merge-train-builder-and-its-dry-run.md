# 0150 — The merge-train builder and its dry run; the switch reads lock

- **Status:** accepted (2026-10-28, T-2987). It implements SPEC-0027 section 4 steps 2-4, its
  "Dry run", and "The switch". Nothing is in force: `scripts/merge_train.toml` says
  `mode = "lock"`, and the merge lock (decisions 0075, 0145) stays the only merge path until
  T-2996's record flips it.
- **Context:** The merge train needs a builder that can be exercised against the live repository
  before any slice pushes or merges. The switch must be impossible for a branch to flip for
  itself, and the id checks must not let a PR judge its own tree (decision 0066).
- **Decision:**
  - `scripts/merge_train.py mode` (`make merge-mode`) reads `scripts/merge_train.toml` from a
    freshly fetched `origin/main` only. A failed fetch, a missing file, a TOML error (duplicate
    keys included), or a missing, non-string or unknown `mode` reads as `lock`.
  - `scripts/merge_train.py --dry-run` (`make merge-train DRY_RUN=1`) collects with
    `merge_ready.verify` (decision 0149), takes up to 5 ready PRs oldest marker first, and builds
    them on `origin/main` in a scratch worktree with `git merge --no-ff`. A PR whose merge bases
    with an earlier member are not all ancestors of `main` (stacked, or criss-cross) is deferred.
    So is one that conflicts only with an earlier member. A conflict with `main` ejects. After
    each merge, `main`'s `tasks.py validate --root` and `check_decisions.py --root` run with
    `python3 -I` from a second worktree. When they fail, the PR just merged added the later
    copy and is ejected; only the colliding ids reach the reason, never file names.
  - The dry run writes nothing to GitHub or any remote. It prints the batch, the deferrals and
    the would-be ejections and unlabels.
- **Consequences:**
  - `tasks.py validate --root <dir>` and `check_decisions.py --root <dir>` exist. `--root` is
    refused for every other `tasks.py` command.
  - Pushing, CI, merging and the lock are T-2994's. Housekeeping of labels on closed PRs is a
    write, so it belongs to the write modes (T-2994, T-2995).
