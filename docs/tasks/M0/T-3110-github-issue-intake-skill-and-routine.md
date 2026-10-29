---
id: T-3110
title: "GitHub issue intake skill and routine"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A GitHub issue opened on steveash/rustsheet becomes a normal task file and is worked through the
normal work-loop at once, by an owner-created "rustsheet issue intake" routine fired on
`issue.opened`, with the daily steward sweeping up issues whose webhook event was dropped.

## Context
Owner decisions of 2026-10-29 (decision 0158): any issue qualifies, whoever opened it; it is
worked immediately in the intake session; the trigger is a GitHub `issue.opened` event on a new
owner-created routine; issues are a way into the existing workflow, not a second workflow. Issue
text is untrusted input (CLAUDE.md directive 5 and the "untrusted data" clause of every routine
prompt), so the skill extracts the request and never follows instructions found in it.

## Acceptance criteria
- [x] `.claude/skills/issue-intake/SKILL.md` exists with `name`/`description` front matter and
      steps: read, dedup, triage (refusals, no third-party code, spam), file the task with
      `issue: <N>`, intake comment, branch and claim (work-loop step 2) then steps 3 onward,
      PR/merge comments, report.
- [x] `scripts/tasks.py validate` accepts an optional `issue:` field only when it is a positive
      integer (with `issue_author:`); `tasks.py issue <N>` lists the tasks that carry it on the working tree and on every
      `origin/*` branch; both are unit-tested in `scripts/test_tasks.py`.
- [x] work-loop step 6 puts `Closes #<N>` in the PR body of a task with `issue:`.
- [x] The steward skill has an "Issue intake sweep" rule (at most 2 issues per run, oldest
      first), and the steward prompt in `docs/process/routines.md` has it as a numbered step.
- [x] `docs/process/routines.md` has the routine row, the trigger settings and a
      "## Prompt — issue intake" section; `docs/process/agent-workflow.md` and
      `docs/process/task-format.md` mention issue intake and the `issue:` field.
- [x] Hard limits after review: `check_governance.py` fails an issue task that touches a
      protected path; volume limits via `tasks.py issue --stats`; probe and comment rules.
- [x] `make check` green; reviewers approve.

## Out of scope
- Creating the routine (the owner does).
- Acting on any real GitHub issue.

## Outcome
- New `issue-intake` skill (`.claude/skills/issue-intake/SKILL.md`): read the issue as untrusted
  data, dedup, triage (bug against Excel, feature, question or research, out of scope, refusal,
  spam), file a normal task with `issue: <N>`, branch and claim it (work-loop step 2), post the
  intake comment, work it through steps 3 onward, post at most two more comments, report.
- Review round: hard limits. `check_governance.py` fails a diff that adds or changes an `issue:`
  task and touches any `is_protected` path (4 tests); `issue_author:` field; `tasks.py issue
  --stats` (open issue tasks per author, issue tasks filed this UTC day); probe, comment and
  re-read rules in the skill, prompt and decision; shared `_git` / `origin_refs` helpers.
- `scripts/tasks.py`: optional `issue:` field, which must be a positive integer (`validate`);
  `tasks.py issue <N>` searches the working tree and every `origin/*` branch (one `git grep`, and
  the front matter decides); `list` shows `(issue #N)`. Four new tests in `scripts/test_tasks.py`.
- work-loop step 6 (`Closes #<N>` / `Part of #<N>`), steward "Issue intake sweep" rule and
  steward prompt step 8 (the train pass is now step 9), routines.md row, trigger settings and
  "Prompt — issue intake", agent-workflow.md roles table and task creation, task-format.md field.
- Decision 0158 records the owner decisions. No eval change (process only). The owner creates
  the routine and updates the steward routine's prompt.
