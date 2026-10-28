# 0151 — Merge-train merge stage for green trains (T-2994)

- **Status:** accepted (2026-10-28, T-2994). It implements SPEC-0027 section 4 steps 1 and 5-7
  for a green train, the write policy of `train` mode, the mode half of the stop checkpoints and
  the squash commit text. Nothing is in force: `scripts/merge_train.toml` says `mode = "lock"`,
  and `--train` stops at once in any other mode, before the lock and before any write.
- **Context:** T-2987 (decision 0150) built and checked trains without writing. The task allows
  a split when the diff grows past size M; the merge stage alone is about that size, so the stop
  label (which needs T-2996's `merge-flip-check`) and the post-squash revert PR and P0 filing
  move to T-3045.
- **Decision:**
  - `scripts/merge_train.py --train` (`make merge-train TRAIN=1`): read the mode (fail closed),
    `merge_lock.py wait --train` capped at the 40-min window, then under the lock: collect,
    build, push a record commit (the train tip's tree, its message listing each PR and head) to
    `merge-train/current` with `--force-with-lease`, renew with `prs`, gate on
    `CI_TIMEOUT + n x MERGE_BUDGET` of the 40-min hold budget, wait for the `ci.yml` run of event
    `push` or `workflow_dispatch` on exactly that branch and sha (one dispatch after 3 min with
    no run), judged by `merge_ready.ci_state`, then squash in order. Each squash is preceded by
    a renewal and GitHub's test merge on the expected `main` with the train's tree, made with
    `expectedHeadSha`, and followed by a check that `main`'s first parent and tree are the
    expected ones. The lock is released in a `finally`, and SIGTERM unwinds into it.
  - Stop checkpoints (right after acquisition, before the CI wait, on every CI poll, before each
    squash) run `git ls-remote` on `main` and re-read the switch only when `main` moved.
  - Any mismatch, a lost lock (exit 4), a red or missing CI run, the budget gate or a refused
    merge stops the sequence: the PRs merged are a prefix of the tested train. A mismatch is
    reported as `P0:`; filing it and the revert PR are T-3045's. Red trains stay T-2998's.
  - Squash text: the PR title with control and format characters stripped, cut to 100
    characters, `@`, `#` and closing keywords neutralised with a zero-width joiner, then
    ` (#k)`; the body holds only well-formed `Co-Authored-By` and `Claude-Session` trailers of
    the PR's commits, de-duplicated.
- **Consequences:**
  - `merge_ready.ci_state` takes the run events that count, so the PR and train rules stay one
    definition; `gh_client` gains `pull_commits`.
  - `train` mode must not be switched on (T-2991) before T-3045 lands: without it a stop label
    is not honoured mid-hold, and a post-squash mismatch is reported but not reverted.
