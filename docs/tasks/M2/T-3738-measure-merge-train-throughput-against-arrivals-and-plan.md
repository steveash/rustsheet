---
id: T-3738
title: "Measure merge-train throughput against arrivals and plan SPEC-0027's revisit (pipelined trains or a larger batch)"
milestone: M2
lane: harness
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A measured comparison of merge-train throughput with PR arrivals, and a planned change if
arrivals exceed what one 5-PR batch per merger run can carry (SPEC-0027 section 3 "Revisit").

## Context
- Found by T-3488. On 2026-11-01 between 09:00 and 13:30 UTC the train merged three batches of
  five (about 10:08-10:20, 11:02-11:12, 12:05-12:09), while ready PRs grew to 21 stranded (marked
  over 2 h earlier) at 13:30 UTC. Train CI takes 10-11 min (decision 0242), twice SPEC-0027's
  assumption of 5.4 min, and a squash in step mode takes about 1-2.5 min, so a hold fits one batch.
- SPEC-0027 section 3 says to revisit (pipelined trains, a larger effective batch, a shorter CI)
  when arrivals stay at 8-10/h or the p90 latency in the "over 6/h" bucket passes 30 min.
- Steward 2026-11-01 (19:47 UTC run): raised to P0 under the `steward` skill's "Stranded in
  `train` mode" rule, which asks for a P0 naming the cause; this task already owns it. `make
  merge-train-stats STRANDED=1` listed **31 stranded PRs** (#2778 ... #2668), up from 21 at
  13:30 UTC. Trains merged batches of five at about 17:30, 18:26 and 20:01 UTC, one batch per
  hour or less, while about 40 PRs were opened between 13:30 and 19:40 UTC. Cause found: arrivals
  exceed one 5-PR batch per hold; the lock was never idle (each hold queued the next train).

## Acceptance criteria
- [ ] From `make merge-train-stats` and the train ref's history: arrivals per hour, PRs merged per
      hour, batches per merger run and p90 marker-to-merge latency for 2026-11-01.
- [ ] If throughput is below arrivals, a spec amendment (or a task to write one) choosing among
      a larger `BATCH_MAX`, pipelined trains or more frequent merger runs, with the arithmetic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing CI wall time (T-3699 to T-3705).
- Changing the stranded threshold (needs a decision record).
