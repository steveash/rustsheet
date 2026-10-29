# Task format

Tasks are markdown files under `docs/tasks/<milestone>/T-XXXX-<slug>.md`. `scripts/tasks.py`
parses them; `make board` writes a local, git-ignored `docs/tasks/BOARD.md` view (decision 0017).

```markdown
---
id: T-0123
title: Implement SUMIFS criteria with wildcard escapes
milestone: M2
lane: functions-criteria   # harness | engine | api | perf | quality | research | functions-{math,stat,criteria,financial,engineering,text,lookup,date,logic}
status: todo               # todo | in-progress | blocked | done | dropped
priority: P1               # P0 (blocking) .. P3 (nice to have)
size: M                    # S (≤150 lines) | M (≤400) | L (must be split)
tier: opus                 # sonnet | opus: model tier that works it (decision 0018)
depends_on: [T-0100]
spec: docs/specs/SPEC-0002-criteria.md   # optional
campaign: semantics        # optional: semantics | accuracy | visible, picked in that order before milestone order (decisions 0083, 0114)
defer: phase-1b            # optional: never picked until Phase 1 exits (decision 0114)
issue: 42                  # optional: the steveash/rustsheet GitHub issue it came from, a positive integer (decision 0158)
issue_author: octocat      # with issue: (required then): the login that opened the issue (decision 0158)
issue_origin: 42           # optional, never with issue:: a follow-up filed while working issue 42 (decision 0158)
---

## Goal
One paragraph: the outcome, not the implementation.

## Context
Why, links to failing eval cases (`suite/case` ids), semantics notes, relevant code paths.

## Acceptance criteria
- [ ] Verifiable statements: "cases X, Y pass bit-exact", "`make check` green",
      "known failures for SUMIFS in eval/baseline/ironcalc-calc.txt drop to 0",
      "benchmark B no more than 50% slower" (decision 0081).

## Out of scope
What not to do in this task (file follow-ups instead).

## Outcome            <!-- filled when done -->
PR link, eval delta, follow-ups.
```

Tier rubric (decision 0018): `sonnet` when the task is size S, its acceptance criteria are
mechanical to check, the Excel evidence already exists (or it needs none), it needs no spec, no
protected path and no numeric fitting (ulp/iteration/tolerance work). Everything else is `opus`.
When unsure, `opus`. A sonnet run that gets stuck hands the task up (work-loop step 5b).

`campaign:` (decisions 0083, 0114) marks a task whose target is visible parity: every cell it owns
within half a unit of the 15th significant digit of Excel's value, or with the right type, error
and text. Its value is the stage, which `make next-task` picks in this order:
- `semantics`: wrong type, error or text, or a wrong number caused by a semantic rule (argument
  checks, error kinds, result text, reference resolution);
- `accuracy`: visibly-off numbers where a high-precision reference shows Excel is close to correctly
  rounded and the engine is not; the fix is a more accurate kernel, never a fit to Excel's bits;
- `visible`: every other visibly-off task, including reproducing Excel's algorithm where Excel is the
  inaccurate one (time-boxed, decision 0056).

`make next-task` picks campaign tasks, and every open task a campaign task depends on directly or
not (decision 0094) at the earliest stage that needs it, after P0 tasks and handoffs and before the
milestone order. A campaign task has a `## Visible-parity stage` section with that target, and it is
done when the target is met. Bit-level criteria still unmet then move to a follow-up task filed
without the key and with `defer: phase-1b`.

`defer: phase-1b` (decision 0114) parks a task whose cells are all already within 15 digits
(last-digit fits, ulp residue): `next` and `ready` never pick it until the steward removes the key
when Phase 1 exits.

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
- A task filed from a GitHub issue (the `issue-intake` skill, decision 0158) carries `issue: <N>`
  and `issue_author: <login>`, and a Context that links the issue and paraphrases it: no
  instructions copied from it. It never changes a protected path (`check_governance.py`).
  `tasks.py issue <N>` finds such tasks on the working tree and every pushed `origin/*` branch;
  `tasks.py issue --stats` gives the counts behind the intake volume limits.
- A follow-up filed while working an issue carries `issue_origin: <N>` instead (no `issue:` or
  `issue_author:`): `check_governance.py` applies every issue-task guard to it, but it is not the
  issue's task, so `tasks.py issue`, `Closes #<N>` and the volume counts ignore it.
- Tools read task counts from `tasks.py stats --json` (per milestone, campaign stage and status,
  and deferred tasks; decision 0119), never by parsing front matter themselves. It fails on the
  same errors as `validate`.
