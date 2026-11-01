---
id: T-3793
title: "New eval cases come with owners: a PR whose added cases hold visibly-off cells that no open task owns fails make check"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A PR that adds or refills eval cases (probes, probes-numeric, gen batches) cannot merge while any visibly-off
Excel cell of those cases has no open task that owns it by `scripts/exit_owners.py`'s rule (a deferred task
counts). So a research or probe PR files the owners of what it finds in the same PR, and gap triage stops being
the place where thousands of cells first get an owner.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main fafb6c6f; `python3 scripts/exit_owners.py --all-off` and
  `--milestone M<k>`). Every non-deferred unowned visibly-off cell of the run came from a PR that added the case
  without an owner the script can see:
  - T-3568 (done) added two probes-numeric cases (its "terms" pair; T-3792 lists them). They hold 7,026
    visibly-off BETA.DIST cells, but no task owned them (its Outcome reads "Eval delta: none in gated suites"). That gap
    triage filed T-3792 for them.
  - T-1925's and T-3532's follow-ups (T-3598, T-3710, T-3711) named their cells as hyphen spans (column C rows 21 to
    27, row 10 columns A to G), which name only their ends (decision 0136). That left 38 M10 exit cells unowned, so M10's exit row
    read them as having no owner. Fixed by hand in the triage that filed this task.
  - T-1683 abbreviated the name of the table-edits "this row, inside data" case (`-this-row-inside-data`), so 11
    M8 exit cells (M8's only unowned ones) had no owner. Fixed by hand in the same triage.
- The visible-parity lock (decisions 0088, 0093, 0219) lets a PR add visibly-off cells in new cases
  (`update-baseline --new-cases`, `visible_parity::judge` `Judgement::Record`), but probes-numeric is not in the lock,
  and nothing asks who owns the new cells. `make unowned-off` and `make exit-owners` exist but gate nothing
  (`STRICT=1` is opt-in).
- Pieces to reuse (directive 3, no second ownership rule): `scripts/exit_owners.py` (`owned`, `case_names`, the
  `--all-off` grouping, deferred handling), `crates/rustsheet-eval/src/baseline.rs` (`new_cases`: cases with no `C`
  line in the merge base's baseline), `crates/rustsheet-eval/src/main.rs` (`listed_new_cases`), `scripts/tasks.py`
  (branch-added tasks against the merge base, as `validate` does for T-3229's shared-cell rule). probes-numeric has no
  `eval/baseline/` file, so "new" for it has to come from git (case files added against the merge base).
- Design questions to settle in the decision record:
  - Where the check runs: inside `rustsheet-eval check` (it has the run's cells) or as a script over
    `eval/runs/latest/report.json` after `eval-gates`. CI and `make check` must both run it, and a run must not
    need a second evaluation of the suites.
  - Which cells count: decision 0188 R1(c)'s visibly-off set (`attributed_off` and `unattributed_off`), plus the
    failing displayed-text items of new M10 cases.
  - Cases a PR only refills (a new oracle sidecar for an existing case) count as new for this check.
- This changes `Makefile`, `scripts/` and `docs/process/` (protected paths, `scripts/check_governance.py`), so the PR
  carries a decision record. The excel-probe skill (`.claude/skills/excel-probe/SKILL.md`, step 4) and the work-loop
  should tell a probe PR to file owners. Those are also protected and are covered by the same decision record.

## Acceptance criteria
- [ ] A decision record in docs/decisions/ states the rule (which cases count as new, which cells, which owners
      count, where it runs), and docs/process/quality-gates.md lists the gate.
- [ ] `make check` (and CI through it) fails when a case the branch adds or refills has a visibly-off Excel cell
      that no open task owns by `exit_owners.py`'s rule. The failure lists each such cell as
      `case!Sheet!A1<TAB>formula<TAB>expected<TAB>actual`. A cell owned only by a `defer:` task passes.
- [ ] Tests: a fixture branch state with a new failing case and no owner fails. The same state with a task naming
      the case passes, and so does one with a task listing the cell in a colon span. A hyphen span naming only the
      ends fails for a middle cell. A case that already had a baseline `C` line before the branch is not checked.
- [ ] On origin/main at the time of the PR, the check passes: no case is new against the merge base.
- [ ] The excel-probe skill's step 4 tells a probe PR to file or amend owner tasks for every visibly-off cell its
      cases add, and to check with `make unowned-off ALL=1`.
- [ ] No eval/baseline or quality/*.json change. `make check` green; reviewers approve.

## Out of scope
- Owning today's unowned cells (gap triage does that; after 2026-11-01's triage no non-deferred cell is unowned
  except the M14 desktop-owned FORECAST.ETS cells).
- Changing the ownership rule itself (decision 0136's hyphen spans, T-3623's colon spans per list block).
