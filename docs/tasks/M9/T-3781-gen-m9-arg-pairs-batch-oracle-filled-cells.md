---
id: T-3781
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 order, rank and paired-regression statistics (SMALL, LARGE, RANK*, PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN, CORREL, PEARSON, COVAR*, SLOPE, INTERCEPT, RSQ, STEYX, FORECAST*, FREQUENCY, PROB, STANDARDIZE, PERMUT, PERMUTATIONA)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of the 31 M9 order, rank and paired-regression statistics below, and every Excel cell the engine
fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main 65c266573; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned
  (`make unowned-off ALL=1` and `make exit-owners MILESTONE=M9` list none of them), and every function is above the
  evidence floor, but most argument-class pairs have no Excel cell. T-3498 (done) ran this batch for the M9
  engineering functions; no task covers the statistical ones. This group has the highest catalog `prior` of the
  M9 statistical gap (SMALL, LARGE and the RANK family 0.5).
- Functions: SMALL, LARGE, RANK, RANK.EQ, RANK.AVG, PERCENTILE, PERCENTILE.INC, PERCENTILE.EXC, QUARTILE,
  QUARTILE.INC, QUARTILE.EXC, PERCENTRANK, PERCENTRANK.INC, PERCENTRANK.EXC, TRIMMEAN, CORREL, PEARSON, COVAR,
  COVARIANCE.P, COVARIANCE.S, SLOPE, INTERCEPT, RSQ, STEYX, FORECAST, FORECAST.LINEAR, FREQUENCY, PROB, STANDARDIZE,
  PERMUT, PERMUTATIONA.
- Pairwise share (unseen pairs) at 65c266573, lowest first: PERCENTRANK.EXC 0.18 (297); PERCENTRANK 0.20 (290);
  SMALL 0.21 (95); COVARIANCE.P 0.21 (96); COVARIANCE.S 0.21 (96); TRIMMEAN 0.21 (96); LARGE 0.22 (94); PERMUT 0.22
  (94); PERMUTATIONA 0.22 (94); PEARSON 0.24 (92); PERCENTRANK.INC 0.24 (277); PROB 0.24 (549); COVAR 0.25 (91);
  QUARTILE 0.25 (91); CORREL 0.27 (88); PERCENTILE.INC 0.27 (88); PERCENTILE.EXC 0.27 (88); SLOPE 0.27 (88);
  QUARTILE.EXC 0.28 (87); QUARTILE.INC 0.29 (86); FORECAST.LINEAR 0.29 (257); FORECAST 0.30 (255); STANDARDIZE 0.30
  (255); PERCENTILE 0.31 (84); INTERCEPT 0.31 (83); STEYX 0.31 (83); RANK.EQ 0.32 (97); FREQUENCY 0.32 (82); RSQ 0.33
  (81); RANK 0.36 (92); RANK.AVG 0.37 (90). 4,336 unseen pairs in all. First unseen pairs: SMALL array/k = (number,
  zero), (number, negative); RANK number/ref = (empty_text, range), (error, range); CORREL array1/array2 = (number,
  fraction), (number, numeric_text).
- Dry run on 65c266573 into a scratch suite that was then deleted: `cd tools && uv run rustsheet-gen --policy
  arg-pairs --milestone M9 --functions` with the 31 names above wrote 48 cases (about 8,700 calls, no pair skipped).
  FREQUENCY spills: generate with `--spill`.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3185 (research, opus), T-3129 and T-3369 (research, sonnet) cover the single argument kinds and the data-array
    kinds of these functions; this batch covers the pairs. Whichever lands second re-reads report.json first.
  - Known residue owners: T-1749 (DEVSQ-style accumulation order, which SLOPE/STEYX/RSQ share), T-3017 (MEDIAN-style
    interpolation, deferred). A last-digit cell of a shape a deferred fit task owns goes to it; a new last-digit shape
    gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep" and lists the R4 guards of
    reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: these functions read their data arrays as aggregates (text, booleans and blanks in a range skipped,
  direct scalars coerced) but their k/quart/percent/x arguments as values, and the paired ones drop pairs with a
  non-number on either side; the error order between the two is undocumented. A new failing cell may be a
  coercion rule, an error-order rule or an accumulation-order residue, and sorting them takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate with `--spill`; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/statistical/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (statistics.md), citing the cells.

## Acceptance criteria
- [x] New gen-m9 cases from `--policy arg-pairs --spill --functions` with the 31 functions above are committed with
      their excel-graph oracle sidecars and 0 fill failures.
- [x] After `make eval`, each of the 31 functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to
      load).
- [x] Failing cells are known failures in eval/baseline/gen-m9.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M9` and `make unowned-off ALL=1` report no new unowned cell
      from the batch.
- [x] Any fit task filed for the batch's cells lists the R4 guards of reviewer-excel-fidelity check 7 (hold-out
      counts, fresh sweep, Gate A envelope result, `FITTED` tag and range, reproducibility floor, no keep-matched
      branch) in its acceptance criteria.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning tasks make them), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- The distribution functions, LINEST/LOGEST/TREND/GROWTH, the FORECAST.ETS family (M14 owners), the criteria
  aggregates (AVERAGEIF(S), MAXIFS, MINIFS: functions-criteria lane), single argument-kind probes (T-3185, T-3129,
  T-3369), and other milestones.

## Outcome
- 48 new gen-m9 cases (8,806 checked cells) from `rustsheet-gen --policy arg-pairs --milestone M9 --spill --functions`
  over the 31 functions, filled on excel-graph (48 sidecars, 0 fill failures). After `make eval` every one of the 31
  has `pairwise` 1.0 in report.json `argument_coverage` (0 unseen pairs; was 0.18-0.37 and 4,336 unseen).
- 8,652 pass. The 154 failing cells are four rules, each recorded in statistics.md and owned by a new sonnet task:
  T-3788 (a text scalar is `#VALUE!` before an error data cell, 30 cells), T-3789 (PERCENTRANK's text or error
  significance before the `#N/A`, 63 cells; settles the T-2256 entry's UNVERIFIED line), T-3790 (TRIMMEAN of a typed
  numeric text or boolean literal, 15 cells), T-3791 (FREQUENCY with no numeric bin is a bin at 0, and an error cell as
  the bins is skipped, 46 cells).
- Eval delta: gen-m9 +48 cases, +8,806 cells checked, +8,652 passing; known failures 2,531 -> 2,685 (154 new-coverage
  failures, 0 regressions); visible-parity lock 3,114 -> 3,268 cells (the same 154). Evidence floor unchanged (0 below).

