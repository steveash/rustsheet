# 0159 — Exit-owner rule: a single A1 name owns a cell only in the block that names its case

- **Status:** accepted (2026-10-29, T-0099). Amends decisions 0096 ("Ownership rule") and 0136.
- **Context:**
  - Under 0096 and 0136 a single A1 name counts "anywhere in the task". `scripts/exit_owners.py` `owned` accepts the
    case named anywhere together with the A1 name anywhere. So a task that lists cells for one case and also names a
    second case owns the second case's cells with the same A1 names, cells it never listed.
  - The gap triage of 2026-10-28 found the leak in T-2425 (FORECAST.ETS, desktop evidence). Its 160-cell line for one
    gen-m9 case made it the owner of `=F.INV(0.5,3,9)` and `=FINV(0.001,123456789.123,4)` in two other gen-m9 cases
    that it names only beside other cells. `scripts/desktop_owned.py` therefore left both cells out of the Phase 1
    rate (decision 0143) as desktop-owned.
  - Under that rule, 107 visibly-off cells counted as owned had no open, undeferred task that lists them beside their
    case (T-0099).
  - Tasks also wrap a cell list onto indented lines of the same bullet (T-3031: the probe's path and `S!A9, A75, ...`
    on one line, `B1, C21, C35, ...` on the next lines). A strictly per-line rule would unown those cells.
- **Decision:**
  - A single A1 name owns a cell only in a **block** of the task's text that also names the cell's case.
  - A block is one line plus its continuation lines. A line opens a new block when any of these holds:
    - it is blank;
    - it is not indented;
    - after its indentation, it starts a list item (`- `, `* `, `1. `) or a table row (`|`).

    Any other indented line continues the block above it. In T-3031's bullet, the indented `B1, C21, C35, ...` line
    continues the line that names the probe, so those cells stay owned. A nested bullet under a case's line is a block
    of its own, so it needs its own case name.
  - The rest of the rule is unchanged:
    - colon spans count only on the case's line (0136);
    - a line naming the case with no A1-shaped token owns the whole case (0096);
    - hyphen spans name only their two ends (0136).
- **Consequences:**
  - `owned` (scripts/exit_owners.py, `names_beside_case` and `blocks`) is the one definition. `make exit-owners`,
    `make unowned-off` and `scripts/desktop_owned.py` all go through it.
  - Cells that lost their only owner are listed, beside their case, in the task that fixes their root cause. T-0099's
    PR body has the table. Planners writing cell lists keep each list in the bullet that names its case.
  - Tests: `scripts/test_exit_owners.py`.
