---
id: T-3498
title: "gen-m9 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the 26 M9 engineering functions below 0.3 pairwise (base conversions, BIT*, BESSEL*, COMPLEX, IM*, CONVERT, ERF, DELTA, GESTEP)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
gen-m9 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the pairwise argument coverage of OCT2HEX, BIN2OCT, DEC2HEX, DEC2OCT, OCT2BIN, BIN2HEX, HEX2BIN, BITXOR, DEC2BIN, BESSELY, BITAND, COMPLEX, HEX2OCT, BESSELK, BITOR, BITRSHIFT, CONVERT, IMDIV, ERF, IMPOWER, BESSELI, BITLSHIFT, IMSUB, GESTEP, BESSELJ and DELTA, with every Excel cell the engine fails owned by a task.

## Context
- Gap triage 2026-10-31 (`make eval` on main 32c4a203, report.json `argument_coverage[].pairwise` and
  `unseen_pairs`; decision 0031 evidence gaps). Every failing Excel cell of these functions is already owned by an
  open task (`make unowned-off ALL=1` lists none), and every function is above the evidence floor, but most
  argument-class pairs (number, zero, negative, fraction, numeric_text, text, empty_text, boolean, error, blank,
  range; `crates/rustsheet-eval/src/argument_coverage.rs`) have no Excel cell at all, so the behaviour on those
  pairs is unverified however many cells pass.
- Pairwise share (unseen pairs) at 32c4a203: OCT2HEX 0.16 (102); BIN2OCT 0.17 (100); DEC2HEX 0.17 (101); DEC2OCT 0.17 (100); OCT2BIN 0.18 (99); BIN2HEX 0.19 (98); HEX2BIN 0.19 (98); BITXOR 0.20 (97); DEC2BIN 0.20 (97); BESSELY 0.21 (95); BITAND 0.21 (96); COMPLEX 0.21 (286); HEX2OCT 0.21 (96); BESSELK 0.22 (94); BITOR 0.23 (93); BITRSHIFT 0.23 (93); CONVERT 0.23 (281); IMDIV 0.24 (92); ERF 0.26 (90); IMPOWER 0.26 (90); BESSELI 0.27 (88); BITLSHIFT 0.27 (88); IMSUB 0.27 (88); GESTEP 0.28 (87); BESSELJ 0.29 (86); DELTA 0.29 (86). 2821 unseen pairs in all.
- T-3279 is the same batch for the M2 functions (its out of scope: batches for other milestones). T-3124 and T-3214 (research) cover the single argument kinds of these parameters; this batch covers the pairs. Open owners for known engineering residue: T-2047 (CONVERT last digit), T-2538 (IMABS/IMARGUMENT ulps), T-1075 (blocked, research): give new cells of those shapes to them.
- Run `make eval` first so the policy reads current unseen pairs, then
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M9 --functions OCT2HEX,BIN2OCT,DEC2HEX,DEC2OCT,OCT2BIN,BIN2HEX,HEX2BIN,BITXOR,DEC2BIN,BESSELY,BITAND,COMPLEX,HEX2OCT,BESSELK,BITOR,BITRSHIFT,CONVERT,IMDIV,ERF,IMPOWER,BESSELI,BITLSHIFT,IMSUB,GESTEP,BESSELJ,DELTA`
  (checked on 32c4a203: the policy honours `--functions`; a dry run for SWITCH,IFS,IFNA wrote 5 cases, 702 cells),
  fill with `uv run rustsheet-oracle fill -b excel-graph --suite gen-m9`, and score.
- The M9 exit counts every M9 function at the `v15_checkpoint` rate (docs/milestones.md, decision 0146) and the
  Phase 1 rate counts gen-* cells (decision 0188), so a failing cell here matters: own each one in the same PR (an existing open task that covers the rule, or a new
  task via `make new-task` in this lane, with `campaign: semantics` and a `## Visible-parity stage` section for a
  wrong type, error or text; case file stem and A1 names in one bullet, decision 0159), and record any new Excel
  quirk in docs/excel-semantics/ citing the cells.
- M9 is the long-tail milestone; this batch ranks after the M5-M7 batches (T-3492 to T-3497).
- Tier: opus: gen-m9 engineering draws have a history of many distinct failure rules (IM* inumber reading T-2541/T-2641, base-conversion places, Bessel orders), so owning the failures takes triage judgment beyond a mechanical run.

## Acceptance criteria
- [ ] New gen-m9 cases from `--policy arg-pairs --functions OCT2HEX,BIN2OCT,DEC2HEX,DEC2OCT,OCT2BIN,BIN2HEX,HEX2BIN,BITXOR,DEC2BIN,BESSELY,BITAND,COMPLEX,HEX2OCT,BESSELK,BITOR,BITRSHIFT,CONVERT,IMDIV,ERF,IMPOWER,BESSELI,BITLSHIFT,IMSUB,GESTEP,BESSELJ,DELTA` with their excel-graph oracle sidecars
      are committed.
- [ ] After `make eval`, each of the 26 functions above has a `pairwise` share of at least 0.9 in report.json
      `argument_coverage`, or the PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m9.txt (`make eval-baseline`, 0 regressions), each
      owned by an open task, and `make exit-owners MILESTONE=M9` and `make unowned-off ALL=1` report no new
      unowned cell from the batch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks), beyond a one-line coercion fix in the
  function's own module that the PR also tests.
- Single argument-kind probes (the research tasks named above) and other functions or milestones.
