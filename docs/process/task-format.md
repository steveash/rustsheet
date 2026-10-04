# Task format

Tasks are markdown files under `docs/tasks/<milestone>/T-XXXX-<slug>.md`. `scripts/tasks.py`
parses them; `make board` writes a local, git-ignored `docs/tasks/BOARD.md` view (decision 0017).

```markdown
---
id: T-0123
title: Implement SUMIFS criteria with wildcard escapes
milestone: M2
lane: functions            # harness | engine | functions | api | perf | quality | research
status: todo               # todo | in-progress | blocked | done | dropped
priority: P1               # P0 (blocking) .. P3 (nice to have)
size: M                    # S (≤150 lines) | M (≤400) | L (must be split)
tier: opus                 # sonnet | opus: model tier that works it (decision 0018)
depends_on: [T-0100]
spec: docs/specs/SPEC-0002-criteria.md   # optional
---

## Goal
One paragraph: the outcome, not the implementation.

## Context
Why, links to failing eval cases (`suite/case` ids), semantics notes, relevant code paths.

## Acceptance criteria
- [ ] Verifiable statements: "cases X, Y pass bit-exact", "`make check` green",
      "known failures for SUMIFS in eval/baseline/ironcalc-calc.txt drop to 0",
      "benchmark B within 5%".

## Out of scope
What not to do in this task (file follow-ups instead).

## Outcome            <!-- filled when done -->
PR link, eval delta, follow-ups.
```

Tier rubric (decision 0018): `sonnet` when the task is size S, its acceptance criteria are
mechanical to check, the Excel evidence already exists (or it needs none), it needs no spec, no
protected path and no numeric fitting (ulp/iteration/tolerance work). Everything else is `opus`.
When unsure, `opus`. A sonnet run that gets stuck hands the task up (work-loop step 5b).

Rules:
- One task produces exactly one PR. If it needs more, split it.
- Acceptance criteria must be checkable by a reviewer agent without guessing.
- `blocked` tasks get a `## Blocked because` section naming the precise need and, for a missing
  capability, the command that proves it is available (decision 0013).
- Create tasks with `make new-task MILESTONE=.. LANE=.. TITLE=..`, never by picking an id by hand.
  It takes the next id after the highest one used in the milestone's block, counting the tasks on
  every pushed `origin/*` branch, so parallel sessions cannot collide (decision 0016). Fill in the
  `TODO(fill)` lines (`tasks.py validate` rejects them), then commit and push the file right away:
  the push is the claim.
