# 0169 — Merge-train step mode: the script waits for the session's ack, then reads each write back

- **Status:** accepted (2026-10-30, T-3077). It implements SPEC-0027 section 4, "GitHub access", the step mode.
- **Context:** T-3001's probe found that a routine session's `gh api` comment, label and unlabel writes are refused
  by the tool-permission layer, while the GitHub MCP tools write. The merger is one long hold: it holds the merge
  lock, a hold budget, the train it built and tested, the bisection state and the PRs merged so far, all in
  memory. Re-running it from the start after each write would rebuild and re-test the train (a CI run per write)
  and could not tell a write it asked for from one it never asked for. The session proxy also refuses branch
  deletion (T-1938), and no MCP tool deletes a branch, so a session cannot perform every write.
- **Decision:**
  - `scripts/merge_step.py` holds the action schema: a closed enum (`comment`, `label`, `unlabel`, `merge`,
    `create_pull`, `delete_branch`, `dispatch`), exact field sets, label names from the train's three labels, and
    the same bounds `gh_client` puts on direct writes. `merge_ready.py mark --step` validates its actions with it.
  - `StepClient` replaces the `gh_client` client of a `--step` run: reads go through the real client; the seven
    write methods each emit one action and refuse any other non-GET request. Before it emits, the script reads the
    state the write changes; the session performs exactly that action and runs `make merge-train-ack SEQ=<n>`
    (`REFUSED=1` when GitHub or its tools refused the write); the script, still running and holding the lock,
    reads the state back.
    - Ack `done`: the state must be exactly the expected result, or the run aborts.
    - Ack `refused`: the state must be unchanged, and the write fails as a direct write would (`GhError`), so
      the hold handles it as today: a refused squash stops the train, a refused branch deletion is logged.
    - No ack within 10 min, or an ack for another sequence number: the run aborts.
    An abort is `StepError` (not a `GhError`, so no step of the hold swallows it): the run exits 2 and releases
    the lock, and nothing after the missing write runs.
  - The comment and PR-body read-back accept the body alone or followed by exactly the attribution footer the
    session's GitHub write path appends.
- **Consequences:**
  - The merger routine runs `make merge-train TRAIN=1 STEP=1` (or `SHADOW=1 STEP=1`) in the background, performs
    each JSON line it prints, and acks it. T-3190 updated the routine prompt in docs/process/routines.md ("Prompt —
    merger") to this flow: run in the background, watch the output, perform each action exactly with the GitHub
    MCP tools, then ack it (`REFUSED=1` when refused or when no tool can perform it, e.g. `delete_branch`).
  - In step mode each squash costs four session round trips (merge, unlabel, comment, branch); `MERGE_BUDGET`
    is unchanged, so a long batch may run past `HOLD_BUDGET` after its CI wait, as a slow merge reply would.
  - Residual risk: the read-back is detective, not preventive. A session that performs a different write (say,
    a merge at another head) is caught only after that write landed. A writer posting an identical comment at
    the same moment would pass for the expected one. The action text carries PR titles that contributors write;
    the schema and the read-back bound what a prompt-injected session can do through the step mode.
