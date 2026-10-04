# Task format

Tasks are markdown files under `docs/tasks/<milestone>/T-XXXX-<slug>.md`. `scripts/tasks.py`
parses them and `docs/tasks/BOARD.md` is generated from them (`make board`).

```markdown
---
id: T-0123
title: Implement SUMIFS criteria with wildcard escapes
milestone: M2
lane: functions            # harness | engine | functions | api | perf | quality | research
status: todo               # todo | in-progress | blocked | done | dropped
priority: P1               # P0 (blocking) .. P3 (nice to have)
size: M                    # S (≤150 lines) | M (≤400) | L (must be split)
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
