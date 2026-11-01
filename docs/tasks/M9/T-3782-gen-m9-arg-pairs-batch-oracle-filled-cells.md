---
id: T-3782
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 coupon and bond functions (COUPDAYS, COUPPCD, COUPDAYBS, COUPNCD, COUPNUM, COUPDAYSNC, PRICE, YIELD, DURATION, MDURATION)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise
argument coverage of the 10 M9 coupon and bond functions below, and every Excel cell the engine fails is owned by a
task.

## Context
- Gap triage 2026-11-01 (`make eval` on main 65c266573; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned
  (`make unowned-off ALL=1` and `make exit-owners MILESTONE=M9` list none of them), and every function is above the
  evidence floor, but most argument-class pairs have no Excel cell. These are the lowest pairwise shares of any
  financial function. T-3498 (done) ran this batch for the M9 engineering functions; no task covers these.
- Pairwise share (unseen pairs) at 65c266573: COUPDAYS 0.13 (630); COUPPCD 0.13 (633); COUPDAYBS 0.14 (622);
  COUPNCD 0.15 (616); COUPNUM 0.16 (611); COUPDAYSNC 0.17 (606); PRICE 0.18 (2080); YIELD 0.19 (2067); DURATION 0.18
  (1493); MDURATION 0.17 (1506). 10,864 unseen pairs in all. First unseen pairs: COUPDAYS settlement/maturity =
  (number, zero), (number, empty_text); PRICE settlement/maturity = (number, empty_text), (number, boolean).
- Dry run on 65c266573 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions COUPDAYS,COUPPCD,COUPDAYBS,COUPNCD,COUPNUM,COUPDAYSNC,PRICE,YIELD,DURATION,MDURATION`
  wrote 115 cases (about 21,700 calls, no pair skipped). Split it into two PRs (the six COUP* functions, then PRICE,
  YIELD, DURATION and MDURATION) if oracle fill or eval time needs it, each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3184 (research, opus) and T-3125 (research, sonnet) cover the single argument kinds of these parameters; this
    batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known residue owners: T-1374 (bond values a few ulps off, deferred) and the DURATION/MDURATION/PRICE last-digit
    sweep task on the 2026-11-01 triage branch (T-3756). A last-digit cell of a shape a deferred fit task owns goes to
    it; a new last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep"
    and lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: the coupon functions truncate dates, validate frequency and basis, and order their errors (settlement
  after maturity, bad frequency, bad basis, a non-number) by rules no doc gives; YIELD iterates. A new failing cell may
  be a coercion rule, an error-order rule, a date-truncation rule or a residue of an existing fit, and sorting them
  takes judgment beyond a mechanical run.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/financial/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (financial.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions COUPDAYS,COUPPCD,COUPDAYBS,COUPNCD,COUPNUM,COUPDAYSNC,PRICE,YIELD,DURATION,MDURATION`
      are committed with their excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 10 functions has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to
      load).
- [ ] Failing cells are known failures in eval/baseline/gen-m9.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M9` and `make unowned-off ALL=1` report no new unowned cell
      from the batch.
- [ ] Any fit task filed for the batch's cells lists the R4 guards of reviewer-excel-fidelity check 7 (hold-out
      counts, fresh sweep, Gate A envelope result, `FITTED` tag and range, reproducibility floor, no keep-matched
      branch) in its acceptance criteria.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning tasks make them), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- The odd-period, discount and T-bill securities (ODD*, ACCRINT*, DISC, INTRATE, PRICEDISC, PRICEMAT, YIELDDISC,
  YIELDMAT, RECEIVED, TBILL*), the annuity functions (T-3779), the cash-flow and depreciation functions (T-3780),
  single argument-kind probes (T-3184, T-3125), and other milestones.
