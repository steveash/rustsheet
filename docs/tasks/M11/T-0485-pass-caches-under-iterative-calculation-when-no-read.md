---
id: T-0485
title: "Pass caches under iterative calculation when no read joins a cycle (calipers pv26_002)"
milestone: M11
lane: perf
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A workbook saved with iterative calculation on (`<calcPr iterate="1"/>`) but with no cycle among the
cells a cached read covers gets the pass caches: the lookup caches (`search_cache.rs`, T-0448) and
the criteria aggregates' range values, match sets and value indexes (`ifs_cache.rs`, T-0449,
T-0465). Its values must stay exactly the same.

## Context
- `Model::pass_cache_safe` (evaluation.rs) refuses every pass cache while `iterating()`. Under
  iteration a read of a cycle member makes every reader on the stack a member too
  (`read_evaluated` and `join_stack` in iteration.rs). A cached read does not go through
  `evaluate_cell`, so it would miss that join.
- calipers `pv26_002` and `pv26_2_002` are saved with `iterate="1"`. Their 1.38M formulas are
  ranks (`COUNTIFS(I$11:I$1450,">"&I11)+1+COUNTIFS(I$11:I11,I12)`) and per-row SUMIFS over formula
  columns, plus SUMPRODUCT (T-0471). They calculate in about 540 s, with or without T-0449 and
  T-0465, because no pass cache runs. Both stay in the suite's `exclude` list.
- A possible approach, to be checked by this task:
  - cache a range only when none of its cells is a cycle member;
  - drop or re-check the caches whenever `join_stack` adds members, since members only grow
    within a round;
  - begin each iteration round (`restart_iteration_round`) with empty caches.
  - Prove that a cached read then makes no reader join that a walk would have made join.

## Acceptance criteria
- [ ] Tests compare walked and cached values under iteration: a cycle through a criteria range or
      a lookup range, cycles that close after a range was cached, and the data-table second round.
- [ ] calipers `pv26_002` release calculation time in the Outcome. If it calculates within 60 s,
      its `exclude` entries go and `rustsheet-eval update-baseline --suite calipers` records it.
- [ ] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- SUMPRODUCT over whole columns (T-0471).
