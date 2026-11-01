---
id: T-3494
title: "gen-m5 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M5 array, subtotal and base-conversion math functions (SUMX2MY2, SUMX2PY2, SUMXMY2, SERIESSUM, MMULT, PERCENTOF, SUBTOTAL, AGGREGATE, BASE, DECIMAL, ROMAN)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m5 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of SUMX2MY2, SUMX2PY2, SUMXMY2, SERIESSUM, MMULT, PERCENTOF, SUBTOTAL, AGGREGATE, BASE, DECIMAL and ROMAN, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: SUMX2MY2 0.31 (83); SUMX2PY2 0.31 (83); SUMXMY2 0.43 (69); SERIESSUM 0.32 (491); MMULT 0.34 (80); PERCENTOF 0.34 (80); SUBTOTAL 0.82 (2); AGGREGATE 0.36 (92); BASE 0.29 (259); DECIMAL 0.43 (69); ROMAN 0.44 (68). 1376 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). T-3493 is the other half of the M5 math functions. The open last-digit task T-2384 (SUMXMY2 accumulation) owns SUMXMY2 ulp cells of that shape: give it any new ones rather than a new task.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M5 --functions SUMX2MY2,SUMX2PY2,SUMXMY2,SERIESSUM,MMULT,PERCENTOF,SUBTOTAL,AGGREGATE,BASE,DECIMAL,ROMAN`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5`, and score.
- M5's exit measures the gen-m5 row at ≥ 99.9% bit-exact (docs/milestones.md, decision 0085; met at 32c4a203),
  and the Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- SUBTOTAL and AGGREGATE take a function number and references: the policy draws their pairs as it draws any other value parameter; a pair that only makes sense with a reference (AGGREGATE ref form) may stay unseen with a reason.
- Tier: size S, the run and the coverage check are mechanical (T-3279 is the same shape and tier), no fitting, fixes out of scope.

## Acceptance criteria
- [ ] New gen-m5 cases from `--policy arg-pairs --functions SUMX2MY2,SUMX2PY2,SUMXMY2,SERIESSUM,MMULT,PERCENTOF,SUBTOTAL,AGGREGATE,BASE,DECIMAL,ROMAN` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 11 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M5` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.

## Outcome
17 gen-m5 cases (`--policy arg-pairs`, 11 functions) oracle-filled with excel-graph; pairwise share is 1.0 (no unseen pairs) for all 11 functions. gen-m5 now 149 cases, 50736 cells, 50710 bit-exact (99.95%). 25 new known failures in eval/baseline/gen-m5.txt, all owned: 21 PERCENTOF cells (Excel `#NUM!` for an error held in a cell reference; literal errors pass through) by T-3547, 2 MMULT cells (Excel `#VALUE!` for a blank first argument beside an error) by T-3548, and the 2 in gen-m5/f5fa1206c7466c62 (A196, A214) by T-3017 (AGGREGATE(12, ...) is MEDIAN of an even count, last-digit ulp: A196 expected 0.6499999999999999, A214 0.6000000000000001). Quirks recorded in numbers.md and arrays.md.
