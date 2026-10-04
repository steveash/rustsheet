# 0018 — Tasks carry a model tier; a sonnet and an opus worker; escalation by handoff

- **Status:** accepted (2026-10-04, owner request; T-0023)
- **Context:**
  - Usage limits bound throughput, and the 3-hourly workers sat idle most of the day. Per-MTok
    prices: Sonnet 5.5 $2 in / $10 out, Opus 5.5 $4 / $20 (Haiku 4.5 $1 / $5, a generation older
    with 200K context). Much of the backlog is small and mechanical; some needs the strongest model.
  - Every task carries a fixed cost (reading CLAUDE.md and skills, `make check`, reviewers, CI), so a
    cheap attempt that fails and is redone costs more than doing it on the right model once.
- **Decision:**
  - Task front matter has `tier: sonnet | opus` (required on open tasks, validated by
    `scripts/tasks.py`); `make next-task TIER=` filters on it and orders handed-off tasks first.
    The rubric (docs/process/task-format.md): `sonnet` only for size S, mechanically checkable,
    evidence already present, no spec, no protected path, no numeric fitting; otherwise `opus`.
  - Two workers replace the functions-lane and critical-lane workers (docs/process/routines.md):
    sonnet hourly, up to 3 sonnet tasks per session; opus every 2 hours, one task, handoffs first,
    and it (with the steward) runs the planner and gap-triager. Both walk the critical-lane list then
    `functions`, one task in flight per lane (busy = branch commit < 2 h, except handed-off branches).
  - Claims: the first commit on `agent/<lane>/T-XXXX-*` is pushed at once, and `make next-task`
    skips a task with such a branch younger than 24 h, so overlapping hourly runs cannot start the
    same task. `next-task` also reads `## Handoff` sections from the pushed branches, so a handoff
    counts as `opus` before it merges. The resuming opus run renames the heading to
    `## Handoff (resumed)` and pushes it, which ends the exemption and makes the branch a claim. A sonnet session starts no new task after 45 minutes.
  - Escalation (work-loop step 5b): a sonnet run that fails `make check` twice on one cause, gets
    changes requested twice, or judges the task beyond it pushes its branch, writes a `## Handoff`
    section, sets `tier: opus` / `status: todo`, and stops.
  - Haiku is not used yet; it is reconsidered only if sonnet-tier data shows a class of tasks it could
    do. Gates are identical for every tier.
- **Consequences:**
  - Cheaper sessions take the mechanical half of the backlog at roughly half the token price, more
    often; Opus time goes to the hard tasks and to finishing handoffs.
  - The owner re-applies routines.md (prompt, model, schedule) after this merges.
  - Check after a week: the steward reports sonnet tasks merged vs handed off. If handoffs exceed
    about a third, tighten the rubric; if almost none, widen it.
