---
id: T-0058
title: "Tick M0's three owner exit boxes with their evidence"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/milestones.md's M0 exit list shows every criterion as it stands, so the three owner boxes left unticked since
2026-10-01 match what has been true for two weeks.

## Context
M0's exit list still had three `[ ]` boxes: oracle setup and calibration, one autonomous work-loop iteration, and
scheduled worker routines. The owner asked for them to be checked against the evidence and ticked (2026-10-18). The
work-loop box cited T-0010, which is "Oracle client: retry transient connection errors", not a work-loop task.

## Acceptance criteria
- [x] Each ticked box cites its evidence; the work-loop box no longer cites T-0010.
- [x] `make hygiene` green; reviewers approve.

## Out of scope
- Any other milestone's criteria.

## Outcome
- Oracle setup: T-0004 is done. docs/research/oracle-calibration.md has 71,765 ironcalc-calc formula cells at 99.96%
  agreement between Excel for the web and the cached values (≥ 99.9%). `rustsheet-oracle fill` has written
  excel-graph sidecars for smoke (5), gen-m2 (30) and probes (866).
- Autonomous work loop: agent sessions have merged task PRs through the work-loop since 2026-10-02 (the first was
  #3, T-0200, 7646134c). Over 1,000 tasks are done this way.
- Scheduled worker routines: the routines in docs/process/routines.md (sonnet A, sonnet B, opus workers every 2 h and
  the daily steward) are enabled and each ran successfully on 2026-10-18. The status.md log records the owner enabling
  them on 2026-10-02 and updating their prompts on 2026-10-04.
