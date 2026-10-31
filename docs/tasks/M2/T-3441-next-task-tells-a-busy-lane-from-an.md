---
id: T-3441
title: "next-task tells a busy lane from an empty one, so a worker launches the planner or gap-triager only when the lane's queue is empty, not while its one in-flight branch holds it"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
When `make next-task` finds nothing to start, it says why. Either every lane asked for is busy, and it names each busy
lane's in-flight branch and the number of ready tasks waiting behind it. Or the queue is empty. The work-loop's pick
step (and the routine text it mirrors) launches the planner or the gap-triager only in the second case, so tasks are
not filed on a lane whose queue is already full.

## Context
- Found 2026-10-31 13:30 UTC: an opus worker launched the planner because `make next-task TIER=opus` printed
  `no ready task` for engine, research, harness and functions. Each of those lanes had a pushed branch with a commit
  in the last 2 hours (engine: agent/engine/T-3426-bound-total-spill-area; research:
  agent/research/T-3334-criteria-operand-probes; harness: agent/harness/T-3359-gate-b-measure and T-3357). Behind
  them, ignoring the busy rule, M2 alone had 7 ready opus engine tasks, 4 research and 2 harness. `tasks.py ready
  --tier opus` printed 63 ready across all lanes. The steward's 05:47 UTC snapshot made the same distinction by hand
  ("0 ready, but only because each lane has an agent branch in flight"). docs/status.md records it.
- Today both cases print the same line. In scripts/tasks.py, `main`'s `next` branch prints `no ready task` whenever
  `available` returns nothing. `available` folds the busy-lane filter (`busy`, decision 0022) into the same list
  comprehension as readiness and claims.
- `.claude/skills/work-loop/SKILL.md` step 1: "If none is ready ... launch the `gap-triager` agent ... otherwise
  launch the `planner` agent". docs/process/routines.md, "Prompt — opus worker": "If no lane has a ready opus task,
  run the gap-triager agent (functions) or the planner agent". Both read the one line.
- `.claude/` and `scripts/` are protected paths: the PR needs a decision record (`make new-decision`) that amends
  decisions 0022 and 0018 on what "no ready task" means. routines.md is the owner's text for the claude.ai routines;
  agents never change routines, so a change to its prompt wording goes to the owner as a "Needs owner" line in
  docs/status.md, not into the routine.

## Acceptance criteria
- [ ] `tasks.py next` with nothing to start prints `no ready task` only when no task passes readiness and claims in
      the lanes asked for. Otherwise it prints one line per busy lane: `lane <lane> busy: <branch> (<n> ready behind
      it)`. Its exit code tells the two apart, and the code is documented in the module docstring and
      `make next-task`'s help text.
- [ ] `tasks.py ready` prints the same busy-lane lines after its count (its exit code is unchanged).
- [ ] scripts/test_tasks.py covers the empty-lane case, the busy-lane case (a fake pushed branch, as the existing claim
      tests do) and a handoff, which is still returned while its lane is busy.
- [ ] The work-loop skill's step 1 launches the planner or the gap-triager only on the empty-queue answer. On the
      busy answer it moves to the next lane in order, then stops with "lanes busy" and the lines above.
- [ ] A decision record states the rule; docs/status.md gets a "Needs owner" line asking the owner to update the two
      worker prompts in claude.ai to match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the one-task-in-flight-per-lane rule itself or the 2-hour busy window (decision 0022).
- The steward's queue-health step (it already counts with and without claims).
