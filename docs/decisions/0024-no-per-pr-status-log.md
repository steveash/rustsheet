# 0024 — PRs no longer append to docs/status.md; union merge for it

- **Status:** accepted (2026-10-05, owner request; T-0033)
- **Context:** every PR was required to append one log line to the end of `docs/status.md`
  (work-loop step 6, agent-workflow step 9). 31 of the 33 PRs merged since 2026-10-04 touched it,
  so any two concurrent PRs conflicted on the same last lines. It was in nearly every conflict that
  day (#53, #56, #61, #62); each one cost a merge, a `make check` and a CI cycle, and the
  resolution was always "keep both lines". The line duplicated the squash commit title and the
  task's Outcome note. Same pattern as BOARD.md before decision 0017.
- **Decision:**
  - PRs do not edit the `docs/status.md` log. The record of merged work is `git log
    --first-parent main` plus each task's Outcome; the daily steward run lists what merged since
    its last run (already step 1 of its prompt), keeps the snapshot table, "Needs owner" and the
    critical-lane line.
  - `.gitattributes` sets `docs/status.md merge=union`, so a branch that still touches it (the
    steward, an older branch) merges with `origin/main` without hand edits.
  - Risk accepted for status.md's in-place sections (snapshot table, `**Critical lane:**` line,
    "Needs owner"): a union merge could keep two versions of an edited line. Only the steward edits
    the snapshot and critical-lane line, and it rewrites them on every daily run; "Needs owner"
    entries are added or removed whole by unblocking agents, which git merges without conflict
    unless two branches touch neighboring lines. The worst case is one stale line until the next
    steward run, which is cheaper than a conflict on every PR.
  - Not applied to `docs/excel-semantics/`: entries there are edited in place, and a union merge
    could silently keep two contradictory versions of a rule. Those conflicts are rare (two PRs on
    the same topic) and worth a human-style look.
- **Consequences:** status.md's log stops at 2026-10-05; history before that stays. GitHub's own
  mergeability check ignores merge drivers, so a PR can still show as conflicting, but the agent's
  `git merge origin/main` (decision 0020) resolves it with no edits.
