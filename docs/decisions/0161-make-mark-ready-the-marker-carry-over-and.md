# 0161 — make mark-ready: the marker, carry-over and the step-mode write fallback

- **Status:** accepted (2026-10-29, T-3040). It amends decision 0149, which left `mark` to T-3040,
  and implements SPEC-0027 section 2 ("The marker", the refusals, "Write fallback",
  "Carry-over"). Like 0149, it is in force for no merge until `scripts/merge_train.toml`'s mode
  says so (T-2996).
- **Context:** T-3001 found that a routine session's tool-permission layer refuses `gh api`
  writes, while the GitHub MCP tools write. The three merge-train labels did not exist on the
  repository (checked 2026-10-29).
- **Decision:** `scripts/merge_ready.py mark --pr N (--review "..." | --carry) [--step] [--by ID]`
  (`make mark-ready PR=<n> REVIEW="..." [CARRY=1] [STEP=1]`).
  - It refuses (exit 1) every case of SPEC-0027 section 2: a draft or fork PR, a base other than
    `main`, the local `HEAD`, `refs/pull/<n>/head` and `head.sha` differing, CI not green by the
    "green" rule, the branch's task file not `status: done` with an `## Outcome`, an applicable
    reviewer without `APPROVE`, and a missing or invalid marker id (`CLAUDE_CODE_SESSION_ID` by
    default). It reuses `verify`'s definitions; the head's changed paths and CI state are one
    function (`head_state`) that both call.
  - `REVIEW` and `--carry` are exclusive: `--carry` takes the verdicts and `prev:` from the newest
    earlier marker by a writer, and only for a head that `Repo.carry_problem` accepts (a merge
    commit, first parent `prev`, second parent on `origin/main`, tree equal to
    `git merge-tree --write-tree` of the parents).
  - A head that already has a writer's marker gets no second comment; only a missing label is
    added.
  - Writes go comment first, then label, so a labelled PR always carries its marker. Direct mode
    writes through `scripts/gh_client.py`, creates any missing train label first (with
    `gh_client.create_label`), then runs `verify` to read the result back (exit 2 on a mismatch).
    `--step` prints the same writes as step-mode JSON actions, one per line: `comment` (`pr`,
    `body`, generated in full by the script) and `label` (`pr`, `label`, and `labels`, the PR's
    whole label set afterwards, for a tool that replaces the set). The session performs them in
    order with the GitHub MCP tools and then runs `make merge-ready-verify`.
  - `make merge-labels` creates the missing train labels. T-3040's run created all three with it
    on 2026-10-29 (`gh api` writes from inside the script ran in that session, unlike T-3001's
    direct `gh api` calls). Should one be deleted, the step-mode label action still recreates
    `ready-to-merge` (the issue-labels endpoint creates a missing label).
- **Consequences:**
  - `scripts/gh_client.py` gains the label read (`label`, None on HTTP 404) and `create_label`.
  - Workers do not use `make mark-ready` yet: the skill and prompt changes are T-2989.
  - T-2995 extends the step-mode schema with the merger's `unlabel` and `merge` actions.
