---
id: T-2591
title: "Probe WORKDAY.INTL/NETWORKDAYS.INTL weekend TRUE vs 2/3, 11.5 vs 12, and an empty weekend argument"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence that pins what probes/weekend-number-coercion could not tell apart, so dates.md "weekend number coercion"
states only what is shown.

## Context
- T-2271's review (PR #1477): the probe starts on 45000, a Wednesday, where weekends 1, 2 and 3 give the same answers, and
  so do 11 and 12. So A1/A3/A14/A16 show TRUE is one of weekends 1-3, not 1 itself, and A6/A19 (11.5) and A8/A21 (1.5)
  cannot tell truncation from rounding. (3.9 and 0.5 do show truncation.)
- An empty weekend argument (`WORKDAY.INTL(45000,5,)`, `NETWORKDAYS.INTL(45000,45030,)`) is unprobed; the engine keeps
  the default weekend (`CalcResult::EmptyArg` in `weekend_mask`), and Excel often reads an empty argument as 0 (#NUM!).

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) with start/weekend pairs where weekend 1 differs from 2 and 3 (e.g.
      starting on a Sunday, 45004), 11 differs from 12, and 1.5/11.5 separate truncation from rounding; plus the two
      empty-argument forms. Literal and cell-read booleans.
- [ ] dates.md updated with what each cell shows; the engine fixed or a follow-up filed if anything differs.
- [ ] `make check` green; reviewers approve.
