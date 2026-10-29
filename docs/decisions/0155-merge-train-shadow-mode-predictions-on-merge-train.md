# 0155 — Merge-train shadow mode: predictions on merge-train/shadow, no lock and no merge

- **Status:** accepted (2026-10-28, T-2995). It implements SPEC-0027 section 4, "Shadow". Nothing is in force:
  `scripts/merge_train.toml` says `mode = "lock"`, and `make merge-train SHADOW=1` stops at once in any mode but
  `shadow`.
- **Context:** before `train` mode is switched on, the merge train has to be shown live without merging anything,
  and its predictions have to be checkable against what the per-PR flow then does (SPEC-0027 "Rollout"). T-2994
  (decision 0151) and T-2998 (decision 0154) built the train's build, CI wait and bisection for `train` mode only.
- **Decision:**
  - `ShadowStage` (scripts/merge_train.py) is `Stage` with every write swapped for a prediction. It takes no lock,
    pushes only to `merge-train/shadow`, merges nothing, and makes one PR-visible write: `ready-to-merge` comes off
    closed PRs. It reuses `Stage.resolve`'s search unchanged (no second bisection). Merging a green prefix is
    predicted, not done, so `main` stays `M0` and the confirmation run and the re-test of the rest run on `M0`.
  - Each round records `shadow-input: <M0> #pr head ...` in its train commits. When the shadow branch tip already
    names the same input, the round is skipped with nothing pushed. A push that loses its lease is another shadow
    build winning the race, so the round is skipped too.
  - The outcome goes into a follow-up `[skip ci]` commit (`merge train shadow prediction [skip ci]`). It has one line
    per PR, `merge`, `defer`, `eject:<ci-red|conflict|id-collision|invalidated>` or `no result` for a PR a stopped
    round did not decide; then `unlabel: #a, ...` (every PR but the deferred ones) and
    `tree-check: match | mismatch | n/a`. The tree check compares the first PR's GitHub test merge with `tree(T_1)`.
    It is `n/a` when `main` moved or the test merge is not on `M0`.
  - A cut-off keeps its search-state record (decision 0154) as the branch tip, with no prediction on top, so the
    next round resumes the bisection.
- **Consequences:**
  - `make merge-train-stats` (T-3076) reads the prediction commits through the commits API and scores them.
  - The step mode for session writes is T-3077. Shadow mode's housekeeping unlabel is its one write, and it goes
    through `gh_client`, so in a routine session it needs T-3077 or an owner-approved session.
  - Re-testing the rest of a red train on `M0` (without the green prefix that `train` mode would have merged) is
    a shadow-only approximation. `merge-train-stats` treats shadow predictions after a bisection as approximate.
