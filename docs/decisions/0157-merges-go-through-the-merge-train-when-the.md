# 0157 — Merges go through the merge train when the switch says so

- **Status:** accepted (2026-10-29, T-2996). It is the policy record of SPEC-0027 ("Decision
  records and governance"). It amends 0028, 0050 and 0075, and leaves 0121 unchanged in substance
  and 0053 dormant. Nothing changes while `scripts/merge_train.toml` says `mode = "lock"`: the
  flips to `shadow` and `train` are T-2989 and T-2991.
- **Context:** The per-PR flow holds the merge lock for one PR's CI run, so at about 4 merges an
  hour the lock is held most of the time and ready PRs wait 30 min or more (SPEC-0027 "Problem").
  The merge train (T-2985 through T-2995) tests a batch of ready PRs on one train branch and
  squash-merges the green ones in order. It needs a policy that says when it, not the per-PR flow,
  merges; what its merges guarantee; and how any agent turns it off at once.
- **Decision:**
  - **The switch** (SPEC-0027 "The switch"). The mode is `scripts/merge_train.toml` on a freshly
    fetched `origin/main`, never the checkout, and the read fails closed to `lock`.
    `make merge-mode` prints it.
  - **0028, amended.** In `train` mode a PR head counts as "tested against current `main`" when
    it was merged into a train built on the current `main`, together with the PRs before it, and
    that train's tree passed CI (SPEC-0027 section 4, step 6). The squash of each PR is checked
    *detectively* (section 5): GitHub guards only the head (`expectedHeadSha`), so each squash is
    compared with GitHub's test merge before it and with `main` after it. A mismatch that landed
    gets a revert PR and a P0. The untested intermediate prefixes are accepted (section 5).
    Readiness verdicts are an attestation by the marking session, not a verification (section 2,
    "Trust"), which is the trust the per-PR flow already places in the merging session. 0028's
    recommendation to turn on "require branches to be up to date" is withdrawn: with it on, the
    train cannot merge PRs 2..n of a batch.
  - **0075, amended.** In `train` mode the merger holds the lock for a batch (`pr: train`).
    Workers hand over with the readiness marker (`make mark-ready`) and do not take the lock. In
    `lock` and `shadow` mode 0075 applies as amended by 0145.
  - **0050, amended.** In `train` mode the steward marks green, reviewed orphaned PRs ready
    instead of merging them. In `lock` mode it may merge or unlabel PRs still labelled
    `ready-to-merge` at once, without the 24 h clock (section 8).
  - **The flip to `lock`** is the one change that may merge with no reviewers and no decision
    record, by any agent, in any mode, under the lock with `expectedHeadSha`. Waiving review works
    in one direction only: turning the train off cannot make a merge less safe than the per-PR
    flow. The rule is mechanical (`scripts/merge_flip.py`):
    - `git diff --raw --no-renames` from the merge base is exactly `M scripts/merge_train.toml`,
      mode `100644` on both sides;
    - both blobs parse with `tomllib`, and their documents are equal except that the head's
      top-level `mode` is the string `"lock"`;
    - `make merge-flip-check PR=<n>` applies the rule to a PR through the files API (exactly one
      file, whose blob is the head's) and the fetched `refs/pull/<n>/head`, refuses a head that
      moved during the check, and prints the head sha to merge with;
    - `check_governance.py` waives the decision-record rule only for a diff that passes the rule.
      Its test-deletion and probe-tag checks still run.

    The P0 task that explains a flip goes in a separate task-only PR, so the flip PR stays one
    file.
  - **The `merge-train-stop` label** means "a flip to `lock` is pending". It is honoured only on
    an open PR that passes `make merge-flip-check`, and removed with a comment from any other PR
    (T-3055 implements it at the train's stop checkpoints and in `make merge-mode`).
  - **Revert PRs** opened by the merger (T-3055) are reviewed and merged under the normal flow,
    with P0 priority. They get no review waiver and no governance waiver.
  - **The flips to `shadow` and `train`** stay fully gated: reviewers, a decision record, and
    architecture and security approval (T-2989, T-2991).
- **Consequences:**
  - `docs/process/routines.md` is the only home of the merger routine's prompt and of the worker
    and steward merge clauses; SPEC-0027 points to it. The owner creates the merger routine and,
    after T-2989 merges, updates the worker and steward routines (docs/status.md, "Needs owner").
  - Rollback: any agent can open the flip PR. Reverting T-2989 and this record restores the per-PR
    texts; the scripts and the fair lock are inert in `lock` mode and can stay.
