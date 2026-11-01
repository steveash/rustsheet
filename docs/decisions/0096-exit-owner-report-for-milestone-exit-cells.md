# 0096 — Exit-owner report for milestone exit cells

- **Status:** accepted (2026-10-21, T-2259). Implements the check that decision 0085 item 3 implies.
  Its ownership rule is amended by decisions 0136 (case.json stems, colon spans) and 0159 (an A1 name owns only in
  the block that names its case).
- **Context:**
  - Decision 0085 item 3: a failing cell of an M5 exit criterion is owned by a task with milestone
    M5, so the pick order reaches it while M5 is current. Nothing checked this; the planner grepped
    `docs/tasks/` by hand.
  - On 2026-10-21 the steward's planner found 13 failing M5 exit cells whose only owner was a done
    task (T-2134): `TRIGONOMETRIC.xlsx` COTH_CSCH_SECH!C51 and C55 and 11 gen-m5 cells. It added
    them to T-2240 by hand.
- **Decision:**
  - `scripts/exit_owners.py` (`make exit-owners MILESTONE=M5`, after `make eval`) lists each failing
    exit cell of the milestone that no open task of it owns, as
    `criterion<TAB>case!Sheet!A1<TAB>formula`, then a count line. It reads only
    `eval/runs/latest/report.json` and `docs/tasks/` (task front matter through `scripts/tasks.py`).
    `--strict` (`STRICT=1`) exits 1 when the count is not 0.
  - Exit cells: every failing cell of the `gen-m<k>` suite's cases; the ironcalc-calc cells of the
    milestone's "Milestone function scope" row less its `excepted` cells (which `make eval` takes
    from `eval/milestones/exit-exceptions.tsv`, decision 0087); and the failing cells of each case
    in the milestone's "Probe milestones" row, or the whole case (`case!*`) when the case fails
    with no cell.
  - **Ownership rule.** A task owns a cell when its front matter has the milestone and an open
    status (todo, in-progress or blocked), and its text contains the case's file stem (the last
    segment of the case id: the gen hash, the xlsx file name such as `TRIGONOMETRIC.xlsx`, or the
    probe name, bounded like a file name) and either the cell's A1 name as a whole word (`A107`,
    `Calc!A107`; `A10` does not name `A1`), or the stem on a line with no A1-shaped token (a task
    that names only the case owns all of it). The second form is per line because report.json
    keeps only failing cells: a task that lists a case's cells, some of which have since passed,
    must not silently own the case's other failures (T-2240 named `TRIGONOMETRIC.xlsx` D51, B15
    and others, but not C51 and C55).
- **Consequences:**
  - The steward and the planner run `make exit-owners MILESTONE=<current>` after `make eval`
    instead of a hand search (since T-2293 the `steward` skill and the `planner` agent say so,
    and give each listed cell an owner: the open task of the milestone that should fix it, or a
    new one). It is a report, not a gate: making it a CI or `make check` step needs
    its own decision.
  - A line that names a case beside any A1-shaped text (`M5`, `LOG10`) does not make the task
    own the whole case. That errs toward unowned: the report may list a cell a task means to own,
    and the fix is to name the cell or the bare case in that task.
  - Cell ranges (`C204..C209`) are not expanded: a task names each cell, or only the case.
  - A probe case in the milestone's row lists all its failing cells, including any that the row
    assigns to another milestone (report.json does not carry the per-cell row).
