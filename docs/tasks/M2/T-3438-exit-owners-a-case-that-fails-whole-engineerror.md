---
id: T-3438
title: "exit_owners: a case that fails whole (EngineError, Panic, did not run) is owned by the block that lists its cells, and a whole-case name counts per block (amend decision 0159)"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/exit_owners.py` gives one answer for a case that fails as a whole (its baseline line is `!EngineError`,
`!Panic`, or the case did not run, reported as `case!*`): the task whose cell list names that case owns the whole-case
failure, wherever in the bullet the case name falls. The rule is written down as an amendment of decision 0159.

## Context
- Found by the planner on 2026-10-31 (`make eval` at 4c8a1a20, then `make exit-owners MILESTONE=M2`). It listed
  `probes/t3266-nested-200-824-stored.case.json!*` and `probes/t3266-nested-200-824-typed.case.json!*` as having no
  M2 owner. T-3338 owns them, in one bullet that wraps over five lines and names 12 `t3266-nested-*` cases, all
  `F ... !EngineError` in eval/baseline/probes.txt. The bullet ends with `: S!A1, S!B1`.
  - `names_whole_case` reads each **line**: it owns a whole case on a line that names the case and has no A1 token.
    So the 10 cases on the bullet's first four lines are owned and the 2 on its last line, beside `S!A1, S!B1`, are not.
  - `names_beside_case` reads each **block** (decision 0159). It never applies here, because a whole-case failure has
    no A1 name (`ExitCell.a1` is None for `WHOLE_CASE`).
  - So the same bullet owns one case and not the next. And a cell list, the form decision 0159 asks planners to write,
    can never own a case that fails whole. The planner works around it for T-3338 in the same PR as this task: it puts
    the two cases on a line of their own with no cell names.
- Decision 0159 says a line naming the case with no A1 token owns the whole case (from 0096). It does not cover a
  whole-case failure next to a cell list. The choice of rule is a governance question, so this task is `opus`, and the
  change needs a decision record (`make new-decision`). Proposed rule, to confirm or replace with reasons:
  1. The whole-case check reads blocks, as `names_beside_case` does, not lines.
  2. A `WHOLE_CASE` exit cell is owned by any block that names its case, with or without A1 names. The failure covers
     every cell, so the task that fixes those cells owns the crash too.
  3. A cell-level failure still needs its A1 name beside the case (unchanged).
- The same `owned` is used by `make unowned-off` and `scripts/desktop_owned.py` (decision 0159 "Consequences"), so the
  change reaches the Phase 1 rate's desktop-owned list. Check that `eval/runs/desktop-owned.tsv` (written by
  `make eval`) changes only where the new rule says it should. Name every changed line in the PR body.
- Code: `owned`, `names_whole_case`, `blocks`, `names_beside_case` in scripts/exit_owners.py. Tests:
  scripts/test_exit_owners.py. `scripts/` is a protected path, which is one more reason for the decision record.
- T-3428 (filed on the unmerged branch agent/harness/T-3359-gate-b-measure) changes which cells exit_owners.py
  lists. This task changes only who owns a listed cell (`owned` and its helpers). If T-3428 lands first, rebase onto
  it; the two changes do not overlap.

## Acceptance criteria
- [ ] A decision record amends decision 0159 with the rule chosen. Decision 0159 is not edited; the new record
      names it in its Status line.
- [ ] scripts/test_exit_owners.py has tests with the T-3338 shape. A bullet wrapped over several lines names cases
      that fail whole, and its last line carries `: S!A1, S!B1`: every case it names is owned. A case that fails whole
      and is named in no block is unowned. A cell-level failure beside another case's cell list is still unowned
      (decision 0159's leak test keeps passing).
- [ ] `make exit-owners` and `make unowned-off ALL=1` on a fresh `make eval` report no newly unowned cell. The PR body
      lists any cell that changes from unowned to owned, or the other way, with the task that owns it now.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing how cells are attributed to functions, or the colon- and hyphen-span rules (decision 0136).
- Rewriting existing tasks' cell lists beyond what a newly unowned cell needs.
