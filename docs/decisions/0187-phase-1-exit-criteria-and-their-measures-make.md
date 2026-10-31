# 0187 — Phase 1 exit criteria and their measures (make phase1-exit)

- **Status:** accepted (2026-10-31, T-3332)
- **Context:** The steward declares `**Phase 1:** exited <date>` when report.json's
  `v15_checkpoint.phase1_met` is true "and the rest of docs/milestones.md's Phase 1 exit holds".
  Nothing measures that rest, and the declaration is costly to undo: it removes every
  `defer: phase-1b` key and unblocks M3, M4 and M13. `phase1_met` covers only the per-function rate and
  coverage. The planner run at 9453cfb5 showed that the rest can go wrong silently. With calipers,
  corpus-bench and poi not fetched, `exit_owners.py --all-off` printed 23 unowned cells; after a fetch it
  printed 0. In the same full run, `--milestone M11` listed six corpus-bench/13894 cells (Sheet1!F20:F25)
  that only an M9 task names, while the M11 exit row read `met` (six visibly off, within its 410-cell
  allowance).
- **Decision:** `make phase1-exit` (`scripts/phase1_exit.py`) prints one verdict per Phase 1 exit
  criterion: `met`, `not met` or `manual`. It reads report.json and the task tree, and it recomputes no
  verdict the report already gives. The criteria map to their measures as follows:
  1. **Run.** The run must be complete: `run.skipped_suites` is empty and `run.dirty` is false.
     Otherwise every other measured line is unreliable, and the overall verdict is `not met`.
  2. **Catalog.** Every Excel function is in the catalog with its parity scope (T-0040): the names of
     eval/catalog/excel-functions.tsv equal the `name:` entries of eval/catalog/functions.yaml, and each
     entry has `scope: in` or `scope: out`, or no scope key, which means `in`.
  3. **Function rate.** Coverage and rate per function: `v15_checkpoint.phase1_met` is true, and both
     `stale_exceptions` and `unchecked_exceptions` are empty. A stale or unchecked exception line means
     the rate was computed against the wrong exception list.
  4. **Owners.** Each remaining failure has a filed task or a `DEVIATION` entry. `exit_owners.unowned_off`
     (the `--all-off` cells) and `exit_owners.unowned` for each Phase 1 milestone list nothing. A cell
     owned only by a `defer:` task counts as unowned. The functions are imported from exit_owners.py, so
     there is one ownership rule.
  5. **Milestones.** For each of M1, M2 and M5–M11, `milestone_exit[].exit` is true. Each row is its own
     line. A row that is met but still has cells the milestone's owner check lists (the M11 case above)
     prints `met` with that count beside it, and the Owners line fails on the same cells. A checker
     reports both facts and does not hide one behind the other.
  6. **Manual.** The prose items checked outside the rows (decision 0146) print as `manual` with their
     docs/milestones.md line: M1's parity bugs and API v0, and M2's one-walker/one-matcher criterion (met
     at 11592ab5, decision 0137).

  The script exits 1 when any measured line is `not met`, and 0 otherwise. `manual` lines never change
  the exit code.
- **Consequences:** The steward declares the exit only when `make phase1-exit` exits 0 on a full
  `make eval` and every `manual` line has been checked by hand. No threshold, allowance or ownership rule
  changes. The progress dashboard can read `--json` (a follow-up).
