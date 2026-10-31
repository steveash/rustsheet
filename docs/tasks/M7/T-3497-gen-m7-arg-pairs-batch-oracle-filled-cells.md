---
id: T-3497
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M7 array-shaping functions (FILTER, SORT, SORTBY, UNIQUE, TAKE, DROP, EXPAND, TOCOL, TOROW, WRAPCOLS, WRAPROWS)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of FILTER, SORT, SORTBY, UNIQUE, TAKE, DROP, EXPAND, TOCOL, TOROW, WRAPCOLS and WRAPROWS, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: FILTER 0.05 (346); SORT 0.29 (517); SORTBY 0.22 (284); UNIQUE 0.32 (248); TAKE 0.25 (271); DROP 0.27 (266); EXPAND 0.16 (612); TOCOL 0.20 (290); TOROW 0.20 (291); WRAPCOLS 0.27 (266); WRAPROWS 0.29 (257). 3648 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). GROUPBY (0.11) and PIVOTBY (0.09) are left out: their 2256 and 4964 unseen pairs mostly involve LAMBDA arguments and need their own batch. T-3496 is the M6 lookup batch.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions FILTER,SORT,SORTBY,UNIQUE,TAKE,DROP,EXPAND,TOCOL,TOROW,WRAPCOLS,WRAPROWS`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`, and score.
- M7's exit is gen-m7 at ≥ 99.9% with 0 spill-extent failures, at visible parity (docs/milestones.md, decision
  0140), and the Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- These functions spill: generate with `--spill` (or check the policy gives each formula an empty footprint) so a `#SPILL!` from a collision is never what gets scored.
- Tier: opus: every result is a spilled array whose footprint the arg-pairs policy has not been used with yet (T-3278 and T-3279 are scalar M2 functions), so the run may need generator judgment before cells are trustworthy.

## Acceptance criteria
- [ ] New gen-m7 cases from `--policy arg-pairs --functions FILTER,SORT,SORTBY,UNIQUE,TAKE,DROP,EXPAND,TOCOL,TOROW,WRAPCOLS,WRAPROWS` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 11 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M7` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.
