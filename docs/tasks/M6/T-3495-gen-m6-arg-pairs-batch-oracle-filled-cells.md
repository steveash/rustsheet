---
id: T-3495
title: "gen-m6 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the 14 M6 date functions (WEEKDAY, NETWORKDAYS.INTL, YEARFRAC, DATEDIF, DAYS360, WORKDAY.INTL, ...)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m6 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of WEEKDAY, NETWORKDAYS.INTL, YEARFRAC, DATEDIF, DAYS360, WEEKNUM, NETWORKDAYS, WORKDAY.INTL, DAYS, EDATE, DATE, EOMONTH, WORKDAY and TIME, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: WEEKDAY 0.19 (98); NETWORKDAYS.INTL 0.21 (571); YEARFRAC 0.21 (285); DATEDIF 0.23 (279); DAYS360 0.23 (279); WEEKNUM 0.25 (91); NETWORKDAYS 0.26 (267); WORKDAY.INTL 0.26 (535); DAYS 0.28 (87); EDATE 0.30 (85); DATE 0.31 (251); EOMONTH 0.33 (81); WORKDAY 0.33 (242); TIME 0.35 (236). 3387 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). T-2864 (research) probes the single kinds (empty text in 20 date parameters, an error in WEEKDAY/WEEKNUM/ISOWEEKNUM); this batch covers the pairs.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M6 --functions WEEKDAY,NETWORKDAYS.INTL,YEARFRAC,DATEDIF,DAYS360,WEEKNUM,NETWORKDAYS,WORKDAY.INTL,DAYS,EDATE,DATE,EOMONTH,WORKDAY,TIME`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6`, and score.
- M6's exit measures gen-m6 the way M5's measures gen-m5 (≥ 99.9% bit-exact, docs/milestones.md, decision 0085),
  and the Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- DATE and TIME are glue functions in the report attribution (a failing cell that also calls another function is blamed on that function), so own their failures by the rule, not by the attributed name.
- Tier: size S, the run and the coverage check are mechanical (T-3279 is the same shape and tier), no fitting, fixes out of scope.

## Acceptance criteria
- [ ] New gen-m6 cases from `--policy arg-pairs --functions WEEKDAY,NETWORKDAYS.INTL,YEARFRAC,DATEDIF,DAYS360,WEEKNUM,NETWORKDAYS,WORKDAY.INTL,DAYS,EDATE,DATE,EOMONTH,WORKDAY,TIME` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 14 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m6.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M6` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.
