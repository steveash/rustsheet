---
id: T-3779
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M9 annuity functions (PMT, PV, FV, NPER, RATE, IPMT, PPMT, CUMIPMT, CUMPRINC, ISPMT, PDURATION, RRI)"
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
argument coverage of the 12 M9 annuity functions below, and every Excel cell the engine fails is owned by a task.

## Context
- Gap triage 2026-11-01 (`make eval` on main 65c266573; report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned
  (`make unowned-off ALL=1` and `make exit-owners MILESTONE=M9` list none of them), and every function is above the
  evidence floor. Yet more than four fifths of their argument-class pairs have no Excel cell. T-3498 (done) ran this
  batch for the 26 M9 engineering functions; no task covers the financial ones, which have the highest catalog
  `prior` of the M9 gap (PMT 0.6, PV and FV 0.4).
- Pairwise share (unseen pairs) at 65c266573: PMT 0.21 (954); PV 0.21 (952); FV 0.20 (964); NPER 0.22 (944);
  RATE 0.17 (1511); IPMT 0.18 (1484); PPMT 0.18 (1484); CUMIPMT 0.18 (1481); CUMPRINC 0.18 (1485); ISPMT 0.22 (567);
  PDURATION 0.21 (286); RRI 0.18 (296). 11,408 unseen pairs in all. First unseen pairs: PMT rate/nper = (number,
  zero), (number, empty_text); RATE nper/pmt = (number, numeric_text), (number, empty_text).
- Dry run on 65c266573 into a scratch suite that was then deleted:
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions PMT,PV,FV,NPER,RATE,IPMT,PPMT,CUMIPMT,CUMPRINC,ISPMT,PDURATION,RRI`
  wrote 128 cases (about 24,800 calls, no pair skipped). That is four times T-3498's batch: split it into two PRs
  (PMT/PV/FV/NPER/RATE/PDURATION/RRI, then IPMT/PPMT/CUMIPMT/CUMPRINC/ISPMT) if oracle fill or eval time needs it,
  each meeting the criteria below.
- Related open tasks; give new cells of their shapes to them rather than filing duplicates:
  - T-3184 (research, opus) and T-3126 (research, sonnet) cover the single argument kinds of these parameters; this
    batch covers the pairs. Whichever lands second re-reads report.json before generating.
  - Known residue owners: T-1528 (PMT/IPMT/PPMT last digits, deferred), T-0458 (PPMT/IPMT ulps, deferred), T-1684
    (PV/FV/NPER outside the fitted sweep), the RATE secant work on T-3579's branch, and the CUMPRINC tiny-rate
    attempts (T-3731 line). A last-digit cell of a shape a deferred fit task already owns goes to that task;
    a new last-digit shape gets a `defer: phase-1b` task, and a numeric-fitting one says "Needs an oracle sweep"
    and lists the R4 guards of reviewer-excel-fidelity check 7 (decision 0188).
- Why opus: RATE, NPER and the cumulative functions iterate or branch on rate sign and type, so a new failing cell
  may be a coercion rule, an error-order rule, an iteration/convergence difference or a last-digit residue of an
  existing fit. Sorting them into owners (and not filing fits blind) takes judgment beyond a mechanical run.
- Steps: run `make eval` so the policy reads current unseen pairs; generate; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`; score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188). Own each failing cell in the same PR: an existing open task that
  covers the rule, or a new task via `make new-task` in this lane (`functions/financial/`). A wrong type, error or
  text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the A1 cells in
  one bullet (decision 0159). Record any new Excel quirk in docs/excel-semantics/ (financial.md), citing the cells.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions PMT,PV,FV,NPER,RATE,IPMT,PPMT,CUMIPMT,CUMPRINC,ISPMT,PDURATION,RRI`
      are committed with their excel-graph oracle sidecars and 0 fill failures.
- [ ] After `make eval`, each of the 12 functions has a `pairwise` share of at least 0.9 in report.json
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
- The cash-flow and depreciation functions (T-3780), the coupon and bond functions (T-3782), the other securities
  functions, single argument-kind probes (T-3184, T-3126), and other milestones.
