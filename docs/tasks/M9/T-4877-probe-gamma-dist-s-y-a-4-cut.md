---
id: T-4877
title: "Probe GAMMA.DIST's y / a * 4 cut below shape 1.01 and pin rule R in a Rust unit test (T-4635 review leftovers)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Rule R of T-4635 rests on evidence at every shape the engine applies it to, a Rust unit test pins its cuts, and the
T-4635 records state its R4 guards and cell counts exactly.

## Context
- T-4635 (PR #3896; statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"): above shape 1 a finite y = x / beta
  is `#NUM!` where y / a * 4 overflows, and to shape 1.1 also where y^a does. Below shape about 1.002 the y / a * 4
  cut comes before y^a's, and no probe has a cell there: the engine (gamma.rs `huge_ratio_num_error`) gives `#NUM!`,
  the Gate A definition gives no value (`HUGE_RATIO_QUOTIENT_SHAPE_MIN`).
- Review findings of #3896 (MINOR): `huge_ratio_num_error` has no Rust unit test; the fresh sweep is counted as 84,
  88 and 104 cells in different places (84 straddling cells, 4 low-shape cells, 16 Cut-sheet cells); the Outcome
  calls the first fill a hold-out although it shaped the candidate list, and the sweep straddles the cut rather than
  being log-uniform.
- Also unprobed: an overflowing y = x / beta above shape 1E300 (`OVERFLOW_SHAPES` in
  tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py stops there, T-4625), where the Gate A definition gives no
  value; a cell there would say whether the overflowing-y `#NUM!` holds at every shape.
- No Gate A batch covers CHISQ.DIST or CHISQ.DIST.RT at a huge x: x = 2y from 1E307 is a literal Excel for the web
  refuses. A way to fill one (or a recorded reason there is none) closes that gap.
- Probe design as in docs/research/t4635 (beta 1E-8, x = y * 1E-8; Excel for the web refuses a typed literal from
  1E307 up).

## Acceptance criteria
- [ ] A probe under eval/suites/probes with oracle values at shapes 1.0001, 1.001 and 1.002, y 4.6E307 to 1.79E308
      (cdf and density), settles which cut holds below 1.01; statistics.md records it, and the engine and the Gate A
      definition follow it.
- [x] A unit test in gamma.rs pins `huge_ratio_num_error` (T-4635's `huge_ratio_cuts`) at shape 1.5 (5E307 no, 7E307 yes), 1.1 (1E280 no, 1E282
      yes), 1.11 (1E306 no) and 1 (1.79E308 no).
- [ ] The gamma.rs doc comment, docs/research/t4635/README.md and the T-4635 Outcome give the same fresh-sweep counts
      and say that the first fill shaped the candidate list.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rule R's verified cuts (T-4635).
