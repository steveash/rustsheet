# 0167 — make merge-train-stats: metrics and shadow agreement from the activity, run and job APIs

- **Status:** accepted (2026-10-29, T-3076). It implements SPEC-0027 "Metrics", "Daily thresholds", "Shadow
  evidence for T-2991" and section 4, "When a shadow prediction agrees". It is read-only: no GitHub or git write
  beyond local `git fetch` of `refs/pull/<n>/head` and `main`.
- **Context:** SPEC-0027 says the shadow evidence comes from each shadow CI run's `headSha`, read through the commits
  API. But T-2995 (decision 0155) writes the prediction into a `[skip ci]` follow-up commit that has no CI run, and
  `merge-train/shadow` and `merge-lock` are force-pushed, so their history is gone from git. The activity endpoint
  (`GET repos/{o}/{r}/activity?ref=...`) lists every push to a ref, force pushes included, with its `after` sha
  (verified 2026-10-29 through the session proxy). It ignores `page=` and pages by an `after=` cursor in its `Link`
  header, which works when re-sent on the `repos/{owner}/{repo}` URL. Workflow run ids are above 10^10, past
  `gh_client.number`'s bound.
- **Decision:**
  - `scripts/merge_train_stats.py` (`make merge-train-stats [SINCE=<date>]`) reads predictions and lock holds from
    the activity of `merge-train/shadow` and `merge-lock` (each push's commit through the commits API), trains from
    the ci.yml runs on the train branches grouped by base and batch, latency and merge-stage runs from merged PRs,
    their head runs and the lock holds, and runner-minutes and queue delay from run jobs. The definitions the spec
    leaves open are fixed in the module docstring.
  - `gh_client` gains `activity` (cursor-paged; `send_cursor` returns the next cursor, validated as base64url
    text), `run_jobs` (run ids bounded by their own pattern), `updated_pulls` (newest update first, stopping at the
    first page past `since`; `pages` takes an `enough` callback), and `event` and `created` filters on
    `workflow_runs`.
  - A prediction its 24 h have not settled is `unsettled` and not scored, like `defer`, `no result` and
    `eject:invalidated`. A predicted head missing from the PR's commits (a rewritten branch) counts as a non-merge
    change. Whether a later head is a clean merge of `main` is `merge_ready.Repo.carry_problem`, the `--carry`
    rule itself; a renumber is a rename under `docs/tasks/` or `docs/decisions/`.
- **Consequences:**
  - T-2991 runs `make merge-train-stats SINCE=<shadow start>` for its bar; the steward's daily thresholds read the
    same command. Train-mode-only sources (ejections, deferrals, batch size and tree checks of `merge-train/current`,
    which records no predictions) are left to T-2990, which measures train mode after 7 days.
  - A 24 h window costs a few hundred `gh api` calls (one per run for its jobs, one per lock push), minutes of wall
    time; nothing in CI runs it.
