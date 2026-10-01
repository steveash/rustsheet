---
name: work-loop
description: One full autonomous iteration on RustSheet - pick the next ready task for a lane, implement it on a branch, pass all gates, get agent reviews, open and merge the PR, update the board. Use when a scheduled routine or the user says "run the work loop for lane X".
---
# Work loop (one task, end to end)

Argument: a lane (`engine`, `functions`, `harness`, `api`, `perf`, `quality`, `research`).
Read `docs/process/agent-workflow.md` once per session if you have not.

## 0. Sync & resume
1. `git fetch origin main && git checkout main && git pull --ff-only`.
2. Resume first: if an open PR or branch `agent/<lane>/*` exists for this lane (check with the
   GitHub tools), continue it (go to step 4/6) instead of starting new work. One task in flight
   per lane.

## 1. Pick
`make next-task LANE=<lane>`. If none is ready: for `functions`, launch the `gap-triager` agent;
otherwise launch the `planner` agent for the current milestone (docs/milestones.md). Then pick
again. If still nothing, write a short note in `docs/status.md` and stop.

## 2. Branch & claim
`git checkout -b agent/<lane>/T-XXXX-<slug>`. Set the task's `status: in-progress` in its file
(commit it as the first commit on the branch).

## 3. Understand before coding
Read the task, linked spec, related code (search for existing helpers — never re-implement),
relevant `docs/excel-semantics/` entries, and failing eval cases (`make eval-show CASE=...`).
If the task needs Excel behavior you cannot evidence, use the `excel-probe` skill.
If the task is bigger than size M, split it (planner) instead of doing it.

## 4. Implement (test-first where possible)
- Add/extend eval spec cases or unit tests that fail first.
- Implement the smallest change that makes them pass. Keep the diff focused.
- `make fmt` then `make check`. Fix everything. Never weaken a gate to get green.
- If eval cells newly pass: `make eval-baseline` and commit `eval/baseline/`.
- If you paid down lint debt / duplication / oversized files: run the script with `--update`.

## 5. Review
Use the `review` skill. Fix every BLOCKING / REQUEST_CHANGES finding, re-run `make check`, and
re-run only the reviewers that did not approve. Max 3 review rounds; if still blocked, set the
task `status: blocked` with a "Blocked because" section, push the branch, open the PR as draft,
and stop.

## 6. PR & merge
1. Mark the task `status: done` (plus a short "Outcome" section), `make board`, commit.
2. `git push -u origin agent/<lane>/T-XXXX-<slug>`; open a PR to `main` using
   `.github/pull_request_template.md`, pasting the eval delta and the review table.
3. Wait for CI (subscribe to PR activity; do not poll with sleep). Fix failures per the `steward`
   skill. When CI is green and all reviewers approve: **squash-merge**. If `main` moved, merge
   `origin/main` into the branch, re-run `make check`, push, wait for CI again.
4. Delete the branch. Append one line to `docs/status.md` (date, task, eval delta).

## 7. Report
Final message: task id, PR link, eval delta, follow-up tasks created.
