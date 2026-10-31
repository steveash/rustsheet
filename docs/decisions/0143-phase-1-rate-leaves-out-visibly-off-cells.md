# 0143 — Phase 1 rate leaves out visibly-off cells owned only by open M14 tasks

- **Status:** accepted (2026-10-28, T-0085).
  Extended by decision 0188 R7 (2026-10-31) to the milestone probe rows.
- **Amends:**
  - decision 0134 item 1: a cell no longer needs its own decision and line in
    `eval/milestones/phase1-exceptions.tsv` to leave the Phase 1 rate when only open M14 tasks own
    it; the hand-kept list stays for cells a decision names;
  - decision 0118 item 3: a function's Phase 1 rate also leaves out these cells.
- **Context:**
  - Decision 0039 rule 3 (docs/milestones.md): a failure whose only open question needs desktop
    Excel evidence has its task in M14 and never blocks the exit of Phase 1. Decision 0114 applies
    it to FORECAST.ETS: excel-graph never computes the family (statistics.md "FORECAST.ETS
    family: excel-graph never computes it"), so its cells are M14 work (T-2425) and "do not block
    the Phase 1 exit".
  - The Phase 1 rate of `v15_checkpoint` (decision 0118) still counted them. On the 2026-10-28
    eval (main 81569726) FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT, FORECAST.ETS and
    FORECAST.ETS.CONFINT had 178, 161, 65 and 58 visibly-off counted cells, so the Phase 1 line
    could never read 506/506 however much other work landed.
  - Decision 0134's list needs a decision naming each cell. T-2425 owns 971 known-failure cells
    (877 at its triage); a decision and a hand-kept line per cell would be a list nobody can
    review, and it would go stale whenever a task file changes.
  - Directive 3: ownership already has one definition, `owned` in scripts/exit_owners.py
    (decisions 0096, 0136, 0138, 0139). The eval is Rust and runs no Python.
- **Options considered:**
  - *Port the ownership rule to Rust.* A second definition of a regex-heavy rule, or a move of
    the first one that every exit-owner report then depends on; large for an S/M task.
  - *A committed list checked to be up to date by `make check`.* The list depends on every task
    file and every baseline: a task-only PR naming a FORECAST.ETS case, or a fix PR that drops an
    F line, would fail the check until it regenerates a file under a protected path, which then
    needs a decision record. Churn with no reviewer value, since the list is mechanical.
  - *Generate the list at each `make eval` from committed files (chosen).* `make eval` runs
    `scripts/desktop_owned.py --out eval/runs/desktop-owned.tsv` and then `rustsheet-eval run
    --desktop-owned`. The list is a pure function of the commit, so it is never stale and never
    committed.
- **Decision:**
  1. A visibly-off, non-deviation Excel cell of the run whose `F` line is in eval/baseline, that
     an open task (status todo, in-progress or blocked) of milestone M14 owns and that no open
     task of any other milestone owns, by exit_owners.py's `owned` rule over the baseline's case
     ids, leaves the Phase 1 count of each function whose counted cells hold it, as decision 0134
     item 2 says for listed cells. Moving a task to M14 is what makes its cells eligible, so that
     move is the reviewable step: it must follow decision 0039 rule 3.
  2. It is reported per function as `desktop_owned` (report.json
     `v15_checkpoint.functions[].desktop_owned`, total `v15_checkpoint.desktop_owned`); report.md
     shows an "M14-owned, left out" column in the functions table and the count per function
     under the Phase 1 line. `rate_cells` and `rate_visible` drop by it. A cell that
     phase1-exceptions.tsv also lists counts once, as `rate_excepted`.
  3. It hides nothing else, as decision 0134 item 3: the cell stays an F line, the gate is
     unchanged, and `cells`, `visible`, the tier, the strict count and decision 0094's
     checkpoint keep it. Cells a non-M14 open task owns are unaffected.
  4. The generated list holds every known-failure cell M14 alone owns, within 15 digits or not, so
     a listed cell that is not visibly off is not stale and draws no warning. `rustsheet-eval run`
     without `--desktop-owned` leaves no such cell out; with it, an absent or malformed file fails
     the run.
- **Consequences:**
  - The four FORECAST.ETS functions meet the Phase 1 rate once every visibly-off cell they count
    is T-2425's (or another M14 task's) alone; a gen-m9 cell a non-M14 task also names still
    counts. The Phase 1 line moves only for functions with M14-owned cells.
  - Agents filing M14 tasks are making a Phase 1 exit call: a task whose open question is not
    desktop evidence belongs in its own milestone (decision 0039). Reviewers check that.
  - `make eval` takes about 7 s more for the list. A steward or agent that runs `rustsheet-eval
    run` directly reports the stricter count unless it passes `--desktop-owned`.
