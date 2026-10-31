---
id: T-3416
title: "make new-task reserves its id atomically on origin, so two sessions or a follow-up filed on a work branch cannot take the same id"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`make new-task` (`scripts/tasks.py new`) claims the id it allocates with one atomic push to `origin`
before it writes the file, and takes the next id when that push is refused, so two sessions that
allocate at the same moment, or a follow-up written on a work branch and pushed hours later, can no
longer end up with two different task files under one id.

## Context
- Decision 0016 made the push of the new task file the claim and called the remaining race "rare".
  It is not rare any more. On 2026-10-31 (origin fetched at about 10:00 UTC), pushed branches add
  seven ids twice, each time for two different tasks:
  - T-3382: agent/functions-text/T-1875-regional-lcid-probes (M10) and
    agent/research/T-2512-incomplete-beta-open-pieces (M9);
  - T-3396: agent/functions-stat/T-2858-t-inv-df1 and agent/functions-stat/T-3350-chisq-inv-huge-df;
  - T-3398 and T-3399: agent/engine/tasks-m2-opus-plan-2026-10-31 (M2) and
    agent/functions-stat/T-2511-inverse-crossing (M9);
  - T-3404: agent/research/T-3334-criteria-operand-probes and tasks/merge-lock-retarget;
  - T-3405: agent/engine/tasks-t3260-followup (M8) and agent/xlsx/T-2913-followup-probe-task (M12);
  - T-3409: agent/functions-stat/T-3350-chisq-inv-huge-df and
    agent/functions/tasks-gap-triage-opus-2026-10-31c.
  Each pair costs the later PR a renumber, a re-review round and a CI cycle (`validate` reports
  "duplicate id" only once the first one merged).
- Two causes, both in `scripts/tasks.py`:
  1. `next_id` takes the most used hundred-block of the milestone, but every milestone's recent tasks
     are now in T-33xx/T-34xx, so all sessions draw from one sequence whatever their milestone.
  2. `git_task_ids` sees only task files already pushed. A follow-up filed inside a work branch
     (work-loop "file follow-ups") is pushed with the work, often hours later, so the window is the
     length of a task, not of one fetch and push.
- A ref push is atomic on the server: pushing a commit to a ref that does not exist creates it, and a
  non-force push of an unrelated commit to an existing ref is refused. A parentless commit made with
  `git commit-tree` (message: id, milestone, title, session) is never a fast-forward of another one,
  so exactly one of two racing sessions wins. `git ls-remote origin '<namespace>/*'` lists the
  reservations without fetching them.
- `scripts/` and `Makefile` are protected paths (`scripts/check_governance.py`), so the PR carries a
  decision record that amends decision 0016 (`make new-decision`).

## Acceptance criteria
- [ ] `tasks.py new` lists the reservation refs with the other taken ids, picks the next id, pushes a
      reservation ref for it, and on a refused push retries with the next id (at most 20 tries, then
      exits non-zero with a message); it writes the file only after a reservation succeeded. The
      namespace and the reason for it (branch list noise, GitHub's reserved `refs/pull/`) are stated
      in the decision record.
- [ ] `--no-fetch` (offline) still works: it skips the reservation and prints a warning that the id
      is unreserved; a push failing for any reason other than "ref exists" (no network, no auth)
      behaves the same way, with the git error in the warning.
- [ ] `scripts/test_tasks.py` covers, against a local bare repository used as `origin`: two
      allocations with the same view of origin get different ids (the second sees its push refused
      and moves on); a reservation without a task file still blocks its id; `--no-fetch` writes the
      file and warns.
- [ ] Reservations do not pile up: `tasks.py prune-reservations` deletes the reservation refs whose
      id is in a task file on `main`, and those older than 30 days whose id is in no task file on any
      `origin/*` branch (both unit-tested on the bare repository). The
      steward skill has no pruning step today, so calling it from the steward is a follow-up task
      filed by the PR that implements this task (`.claude/` is protected), named in the Outcome.
- [ ] Decision record amending 0016 committed; docs/process/task-format.md's `make new-task` rule
      states that the id is reserved at allocation (one sentence).
- [ ] The seven collisions listed above are left to their PRs (each renumbers on its own branch);
      the Outcome lists which of them are still open at merge time.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Renumbering the existing duplicates on other agents' branches.
- Changing the block-per-milestone id scheme itself (cause 1); the reservation makes it harmless.
- Merge-train or merge-lock changes (SPEC-0027).
