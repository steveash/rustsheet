---
id: T-3780
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 cash-flow, rate-conversion and depreciation functions (NPV, XNPV, IRR, XIRR, MIRR, FVSCHEDULE, EFFECT, NOMINAL, DOLLARDE, DOLLARFR, SLN, SYD, DB, DDB, VDB, AMORLINC, AMORDEGRC)"
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
argument coverage of the 17 M9 cash-flow, rate-conversion and depreciation functions below, and every Excel cell the
engine fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main 65c266573; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned
  (`make unowned-off ALL=1` and `make exit-owners MILESTONE=M9` list none of them), and every function is above the
  evidence floor, but most argument-class pairs have no Excel cell. T-3498 (done) ran this batch for the M9
  engineering functions; no task covers these.
- Pairwise share (unseen pairs) at 65c266573: NPV 0.45 (66); XNPV 0.28 (263); IRR 0.29 (86); XIRR 0.20 (290);
  MIRR 0.20 (289); FVSCHEDULE 0.19 (98); EFFECT 0.29 (86); NOMINAL 0.27 (88); DOLLARDE 0.20 (97); DOLLARFR 0.17 (100);
  SLN 0.18 (298); SYD 0.16 (611); DB 0.17 (1007); DDB 0.17 (1000); VDB 0.16 (2134); AMORLINC 0.22 (1980);
  AMORDEGRC 0.23 (1951). 10,444 unseen pairs in all. First unseen pairs: IRR values/guess = (number, zero),
  (number, negative); XIRR values/dates = (number, negative), (number, numeric_text); VDB cost/salvage = (number,
  empty_text), (number, boolean).
- Dry run on 65c266573 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions NPV,XNPV,IRR,XIRR,MIRR,FVSCHEDULE,EFFECT,NOMINAL,DOLLARDE,DOLLARFR,SLN,SYD,DB,DDB,VDB,AMORLINC,AMORDEGRC`
  wrote 109 cases (about 20,900 calls, no pair skipped). Split it into two PRs (cash flows and rate conversions,
  then depreciation) if oracle fill or eval time needs it, each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3184 (research, opus) and T-3126 (research, sonnet) cover the single argument kinds of these parameters; this
    batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known residue owners: T-1686 (IRR/XIRR caps and fallback), T-3246 and T-3411 (IRR), T-1374 (depreciation ulps,
    deferred), T-1528 (NPV last digits, deferred). A last-digit cell of a shape a deferred fit task already owns goes
    to it; a new last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle
    sweep" and lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: IRR, XIRR and MIRR iterate over array arguments whose text, blank and error elements Excel treats by
  its own rules; AMORLINC/AMORDEGRC and VDB branch on dates, basis and switch flags. A new failing cell may be a
  coercion rule, an error-order rule, an iteration difference or a residue of an existing fit, and sorting them
  takes judgment beyond a mechanical run.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/financial/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (financial.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions NPV,XNPV,IRR,XIRR,MIRR,FVSCHEDULE,EFFECT,NOMINAL,DOLLARDE,DOLLARFR,SLN,SYD,DB,DDB,VDB,AMORLINC,AMORDEGRC`
      are committed with their excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 17 functions has a `pairwise` share of at least 0.9 in report.json
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
- The annuity functions (T-3779), the coupon and bond functions (T-3782), the other securities functions, single
  argument-kind probes (T-3184, T-3126), and other milestones.
