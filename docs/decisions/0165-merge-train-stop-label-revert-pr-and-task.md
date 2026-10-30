# 0165 — Merge-train stop label, revert PR and task filing (T-3055)

- **Status:** accepted (2026-10-29, T-3055). It completes SPEC-0027 section 4 on top of decisions
  0151 (the merge stage) and 0157 (the switch and the stop label). Nothing is in force while
  `scripts/merge_train.toml` says `mode = "lock"`.
- **Context:** Decision 0151 split T-2994: a mismatch was only reported (`P0:` in the log), the
  stop checkpoints read only the mode, and the P1 of a red train that did not reproduce (T-2998)
  was only held in `Stage.tasks`. Decision 0157 defines the `merge-train-stop` label and leaves
  its implementation to T-3055.
- **Decision:**
  - Every stop checkpoint of `--train` first lists the open PRs labelled `merge-train-stop`,
    lowest number first, and runs `merge_flip.pr_problems` (`make merge-flip-check`) on each: the
    first that passes stops the hold at once, with nothing more merged. Each one before it that
    fails loses the label, with the comment `merge train: stop label removed: not a valid flip PR
    (...)`. A check that could not run git honours the label and removes nothing, because stopping
    is always safe. Shadow mode neither honours nor removes the label.
  - `make merge-mode` prints `train (stop pending: #n)` when a train would honour PR n's label. It
    only reads.
  - The post-squash check now reads `S_k` from the merge reply (or `merge_commit_sha` after a lost
    reply). A squash with another parent or tree than the tested one is reverted: `git revert
    --no-edit S_k` on a scratch worktree at the current `main`, pushed to
    `merge-train/revert-<S_k>`, and a PR opened whose body the script writes (squash, expected and
    actual tree and parent). The merger never merges it. A good `S_k` with a foreign commit on top
    is a P0 with no revert.
  - The P0 (named after the PR, with the revert PR or why there is none) and any P1 go in one
    task-only PR per hold, from `merge-train/tasks-<first id>`. The files come from `main`'s
    `tasks.py new` (M2, `harness`, `opus`, size S), with a script-written body, so no PR text
    enters them. The lock is released after the filing, in the same `finally` as before.
  - After each squash, the merged PR's branch is deleted when it is in this repository. A refusal
    (the session proxy's 403, T-1938) is logged and the train goes on.
  - `gh_client` gains `create_pull` and `delete_branch`. Branch names in URL paths are refused
    when a segment is empty, starts with `.` or ends with `.lock`, and `main` is never deleted.
- **Consequences:**
  - The write path is still `gh_client` (`gh api`). T-3001 found that a routine session's
    permission layer may refuse `gh api` writes; the step mode (T-3077) must therefore also carry
    the writes added here (unlabel `merge-train-stop`, open a PR, delete a branch) before `train`
    mode is switched on (T-2991). T-3077's context now says so.
  - The section 4 write policy gains the `merge-train/tasks-*` pushes and the two PR openings.
  - T-3156 moved the mismatch response and the task filing (`respond`, `open_revert`, `file_tasks`,
    `write_task` and the types, constants and helpers they use) unchanged into `scripts/merge_train_filing.py`, which
    `merge_train.py` imports and which imports nothing from the train, to bring `merge_train.py`
    back under the file-size limit. Nothing this record decides changes; the tests changed only
    their import paths.
