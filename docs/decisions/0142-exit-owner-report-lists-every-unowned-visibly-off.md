# 0142 — Exit-owner report lists every unowned visibly-off cell with --all-off

- **Status:** accepted (2026-10-28, T-0086). Extends decision 0096's exit-owner report (with
  0132's `--unattributed` listing and the ownership rule of 0136, 0138 and 0139).
- **Context:**
  - docs/milestones.md's Phase 1 rule "each remaining failure has a filed task" needs a list of
    every visibly-off Excel cell no open task owns. `exit_owners.py` had `--milestone` (only the
    milestones with a `milestone_exit` row) and `--unattributed` (decision 0132); nothing listed
    the cells attributed to a function, most of the gap. The T-0090 triage built that list by
    hand, re-applying closeness.rs's visibly-off rule in Python.
  - Directive 3: the visibly-off rule and decision 0118's attribution must have one definition.
- **Decision:**
  - The eval writes `attributed_off` into report.json: the in-scope, visibly-off, non-deviation
    Excel cells with a sole call (decision 0118), each with case, cell, function, formula,
    expected and actual. `attribution::off_excel_cells` is the one filter behind both it and
    `unattributed_off`, so the script repeats no rule.
  - `scripts/exit_owners.py --all-off [--function NAME] [--strict]` (`make unowned-off ALL=1
    [FUNCTION=NAME]`) lists the cells of `attributed_off` (grouped by function), then
    `unattributed_off`, then the probe-milestone cells of every milestone that neither holds, that
    no open task owns by the ownership rule, as
    `group<TAB>case!Sheet!A1<TAB>formula<TAB>expected<TAB>actual`, then per-group and total counts.
  - A cell owned only by a task with `defer:` is unowned: it is listed and counted, and its group
    reads `<group> (deferred)`.
  - `--function` keeps one group: a function name, `unattributed` or `probe M<k>`; a probe case that
    failed with no cell rows is listed once with the key `*`.
  - It gates nothing (`--strict` exits 1 when any cell is listed).
- **Consequences:**
  - The steward and the gap-triager can run `make unowned-off ALL=1` after `make eval` instead of a
    hand triage; T-0087 files owners for what it lists. Case-name checks are cached per case so a
    full report runs in about 30 s. Tests: `scripts/test_exit_owners.py` and
    `attribution.rs`'s tests.
