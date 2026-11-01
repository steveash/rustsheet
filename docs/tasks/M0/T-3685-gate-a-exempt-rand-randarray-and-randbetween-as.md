---
id: T-3685
title: "Gate A: exempt RAND, RANDARRAY and RANDBETWEEN as composite once T-3631's invariant-cell probe lands (decision 0228)"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-3551]
---

## Goal
RAND, RANDARRAY and RANDBETWEEN have `composite` lines in `eval/accuracy/exempt.tsv`, naming the oracle-filled
invariant-cell probe that drives them, so Gate A no longer counts them as missing tables.

## Context
- Decision 0228 (T-3551) rules the three volatile functions `composite`: a draw has no reproducible value to tabulate,
  and they are judged on the cells that do not depend on the draw (range, spill shape, integer-ness, degenerate ranges,
  argument errors), the evidence decision 0225 uses for Gate B.
- T-3551 left the lines out because that probe and decision 0225 are not yet on main (T-3631's branch). A protected
  file must not cite a record main does not hold.
- Not ready until T-3631 merges: its task file is not on main yet, so `depends_on` cannot name it. Pick this task up
  only once main holds decision 0225 and T-3631's probe case.

## Acceptance criteria
- [ ] Once T-3631 has merged, `exempt.tsv` gains the three `composite` lines, each naming the probe case T-3631 adds and
      decision 0225 (the change is covered by decision 0228; no new record is needed unless the ruling changes).
- [ ] `rustsheet-eval accuracy` reports `missing tables` lower by 3, with no stale exemption.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change.
