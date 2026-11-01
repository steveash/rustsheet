# 0093 — The visible-parity lock is lowered daily by the steward, not by each fix PR

- **Status:** accepted (2026-10-21, T-2245; *amended by decision 0217: the lowering is per
  case*; the owner chose this option in session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd). Amends decision 0088.
- **Context:**
  - Decision 0088 made `rustsheet-eval check` hold the visibly-off count to
    `eval/baseline/visible-parity.json` exactly: a drop failed the gate as "stale", so every PR
    that fixed a visibly-off cell committed a new number into the same one-line file.
  - Any two such PRs in flight then conflict textually. GitHub runs no CI on a conflicting PR, so
    the author gets no event, and a worker whose session budget ran out never comes back. On
    2026-10-21 05:30 UTC, 21 PRs sat idle. 7 of them conflicted on this file, and 6 conflicted on
    nothing else. #1393 (T-2167) already conflicted when it was opened, so CI never ran on it at all.
  - A git merge driver cannot help: GitHub decides mergeability without local drivers.
- **Decision:**
  - `check` fails only a rise above the lock. A count below the lock passes; the gap is slack.
  - `update-baseline` (`make eval-baseline`) writes the lock only for a rise: one that new cases
    account for, or any rise with `--allow-regressions` (which still needs a decision record). It
    never writes a drop, so a PR that only fixes cells leaves the file out of its diff.
  - `update-baseline --lower-visible-lock` (`make eval-lower-visible-lock`) also records a drop.
    The steward runs it once a day, in its own PR, and agents do not run it in task PRs.
- **Consequences:**
  - Fix PRs no longer conflict with each other on the lock. A probe PR that adds visibly-off cells
    beyond the slack still writes the lock, so probe PRs can still conflict with each other on it;
    the resolution is to take `main`'s side and re-run `make eval-baseline`.
  - For up to a day, a PR could make up to the slack's worth of already-failing cells visibly off
    without failing the gate, if other PRs fixed as many. Cells that used to pass are still
    caught one by one by the per-cell baselines; this only loosens the 0088 ratchet on cells that
    were already failing. The Excel-fidelity reviewer still reads the report's "Visibly off"
    section, and the steward's daily lowering takes the slack back.
  - When resolving an open PR's conflict on the lock, take `main`'s side.
