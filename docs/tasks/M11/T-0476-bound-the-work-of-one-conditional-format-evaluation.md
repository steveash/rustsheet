---
id: T-0476
title: "Bound the work of one conditional-format evaluation (directive 5)"
milestone: M11
lane: perf
status: todo
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
- [ ] A style read of corpus-bench 41978 returns in under 1 s; the bound and its constant are
      documented, with the DEVIATION entry if styles change past it.
- [ ] Tests for a rule area past the bound, and for the styles within it being unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
