---
id: T-3323
title: "Incomplete beta bfrac bit-level scores: the (100, 120) run's 185 of 1,024 and r2-bratio's 146/685 of 1,316 once the large-shape density lands (T-2508 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2508, T-3244]
---

## Goal
The bfrac port reaches the bit-level scores T-2508 carried: at least 185 of 1,024 bit-exact on the (100, 120) run
(probes-numeric t2464-w2f-xruns-1, the BETA.DIST(.,100,120,TRUE) cells) and at least 146 bit-exact and 685 within
15 digits of r2-bratio's 1,316 bfrac cells.

## Context
- T-2508 (Outcome) ported bfrac with the engine's T-2507 density: 47 of 1,024 on the (100, 120) run (23 before). The
  engine's density is only 33 of 1,024 bit-exact on that run's FALSE cells, and w4's 185 used Excel's own pdf read off
  the sheet, so the score waits on the large-shape density (T-3244, a + b - 2 >= 90). An x87-extended fraction scored
  42, no better than the double one (statistics.md "Incomplete beta L140 and bfrac").
- r2-bratio's scores were not measured on the engine in T-2508: score docs/research/t2464/r2-bratio's bfrac cells
  (`score.py`, cells.tsv) against the engine (`rustsheet-eval run` on a case, or a test over the cells).
- `bratio/tests.rs` `fixture_ported_branches_score` ratchets bfrac at 39 of 82 fixture rows: raise it when this lands.

## Acceptance criteria
- [ ] The (100, 120) run scores at least 185 of 1,024 bit-exact (`rustsheet-eval run --suite probes-numeric
      --filter t2464-w2f-xruns-1`), or the Outcome states the measured score and which density cells cap it.
- [ ] r2-bratio's bfrac cells are scored on the engine and the numbers recorded in statistics.md.
- [ ] No gated known-failure line is added; `make check` green; reviewers approve.

## Out of scope
- The fraction's own arithmetic (T-2512).
- basym (T-3322).
