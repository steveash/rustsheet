---
id: T-3686
title: "Gate A generator: 2-D array literals, so MDETERM, MINVERSE and MMULT get tables in place of their composite exemptions"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3551]
---

## Goal
The Gate A generator writes 2-D array literals (`{a,b;c,d}`), and MDETERM, MINVERSE and MMULT get oracle-filled `s1`
tables in place of their `composite` exemptions.

## Context
- Decision 0228 (T-3551) exempts MDETERM, MINVERSE and MMULT as `composite` only because `literal.Arg`'s array kind
  (T-3586) writes one row. They have real floating-point kernels (arrays.md "MDETERM: partial-pivot LU, no singularity
  cutoff", "MINVERSE: singularity rule"), so a table is the better evidence.
- MINVERSE and MMULT return arrays: a table point is one cell, so read one element (for example through INDEX) and say
  how in the definition; the generator writes `=FUNC(args)` today, so this needs a small extension too.
- Removing the three `exempt.tsv` lines changes a protected path: a decision record is needed.

## Acceptance criteria
- [ ] `literal.Arg` and `parse_formula` handle 2-D array literals, with tests; existing tables regenerate byte-identical.
- [ ] Definitions and filled `s1` tables for MDETERM, MINVERSE and MMULT; their `exempt.tsv` lines are removed under a
      decision record; `rustsheet-eval accuracy` stays at 0 failing with each failing point owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change.
