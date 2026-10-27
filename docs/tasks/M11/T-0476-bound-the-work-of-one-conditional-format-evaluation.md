---
id: T-0476
title: "Bound the work of one conditional-format evaluation (directive 5)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0468]
---

## Goal
One conditional-format evaluation (`evaluate_conditional_formatting`) does bounded work, whatever
the workbook's rules and areas, so a style read of an untrusted workbook cannot hang its host.

## Context
SPEC-0021 "Resource limits": evaluating conditional formats on the first style read (T-0468)
removes the work from load and `evaluate()`, but a style read still costs the whole evaluation.
corpus-bench 41978 costs about 190 s: expression rules over near-million-row areas, each cell's
formula building a range array.

Options:
- a budget of formula evaluations or array elements per evaluation, past which the remaining
  cells get no conditional result;
- evaluating only the rules whose areas hold the queried cell, per read.

A budget changes styles past it, so it needs a DEVIATION entry (decision 0014,
docs/excel-semantics/README.md) and a probe-free justification (directive 5 wins).

## Acceptance criteria
- [x] A style read of corpus-bench 41978 returns in under 1 s; the bound and its constant are
      documented, with the DEVIATION entry if styles change past it.
- [x] Tests for a rule area past the bound, and for the styles within it being unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- The second option of the Context, which needs no budget and no DEVIATION: a style read evaluates
  only the rules whose areas hold the cell read, in priority order, for that cell. The results are
  what the evaluation of every cell writes for it, since each rule's per-cell decision depends
  only on the cell and the rule's statistics.
  - `CfState` (cf_styles.rs) replaces `Model::cf_cache`/`cf_stale`/`cf_runs` with one field.
  - Each rule's write loop goes through `CfState::visited`: all of its areas, or while one cell is
    evaluated, that cell once per area holding it, so overlapping areas apply twice as before.
  - The statistics (`collect_numeric_values`, `value_counts`, shared by the unique and duplicate
    rules, now one `apply_cf_value_count`) are computed once per round, over the areas cut at
    the used area (`cf_clipped`; every cell left out is empty and counts for nothing).
- corpus-bench 41978 (release, after `evaluate()`):
  - the first reads of 8 cells, rows 1 to 1,048,000 of the rule columns included, take 322 ms;
  - a read of all 2,405 used cells takes 152 ms;
  - before, the first read took about 190 s.
- Bound: a read costs the rules over its cell, each one formula evaluation, under the
  evaluator's own limits, plus once per round the statistics over the used area.
  `evaluate_conditional_formatting()` still evaluates every cell, for a caller that asks.
- Tests:
  - `on_first_read.rs`: each cell read alone, read in order, and after an eager evaluation get
    the same styles, with overlapping and repeated areas, stop-if-true rules between others,
    statistics rules and cells past the used area. Statistics are recomputed each round.
  - rustsheet-xlsx `test_conditional_formatting.rs`: the same for every cell of
    cf_tests.xlsx (Excel-authored, every kind of rule), and 3 rows and columns past it.
  - Mutations: no cell written for a read, and statistics kept across rounds, each fail a test.
- SPEC-0021 "Resource limits" and "When the evaluation happens" revised.
