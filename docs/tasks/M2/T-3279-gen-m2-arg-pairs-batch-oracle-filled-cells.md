---
id: T-3279
title: "gen-m2 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of the M2 functions (MID, LEFT, RIGHT, ROUND*, INDEX, HLOOKUP, VLOOKUP, MATCH, IF, IFERROR)"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3278]
---

## Goal
gen-m2 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs` (T-3278) that raises the
pairwise argument coverage of the M2 functions, with every Excel cell the engine fails owned by a task.

## Context
- M2 pairwise shares at 73b18722 (report.json `argument_coverage[].pairwise`): MID 0.399, LEFT 0.43,
  RIGHT 0.455, ROUNDDOWN 0.496, ROUNDUP 0.504, ROUND 0.521, INDEX 0.559, HLOOKUP 0.578, VLOOKUP 0.619,
  MATCH 0.664, IF 0.675, IFERROR 0.744; COUNTIF, COUNTIFS, SUMIF and SUMIFS are at 1.0.
- Run `make eval` first so the policy reads current unseen pairs, then generate with `--milestone M2`,
  fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite gen-m2`, and score.
- M2's exit needs gen-m2 with no visibly-off cell (docs/milestones.md, decision 0135), so a failing
  cell here holds M2 open until fixed: own each one in the same PR (an existing open task that covers
  the rule, or a new task via `make new-task`; case file stem and A1 names in one bullet, decision
  0159), and record any new Excel quirk in docs/excel-semantics/.

## Acceptance criteria
- [ ] New gen-m2 cases from `--policy arg-pairs` with their excel-graph oracle sidecars are committed.
- [ ] After `make eval`, every M2 function listed above has a `pairwise` share of at least 0.9, or the
      PR lists the pairs left unseen and why (a hard domain, a pair Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m2.txt (`make eval-baseline`, 0
      regressions) and `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine for the cells this batch finds (the owning tasks).
- Batches for other milestones.
