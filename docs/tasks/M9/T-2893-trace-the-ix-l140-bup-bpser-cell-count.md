---
id: T-2893
title: "Trace the ix L140 bup+bpser cell count (model 1,620, ported dispatch 1,383) and score T-2506's criterion 2 on the model's set"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2506]
---

## Goal
T-2506's criterion 2 ("the ix L140 cells score at least 1,011 bit-exact of 1,620") is either measured on the same
1,620 cells the model was scored on, or the difference between those cells and the engine's L140 set is explained
cell by cell, so the engine's L140 port is judged against the model on one population.

## Context
- T-2506 (docs/research/t2506/RESULTS.md) scored the engine's K(a, b, x, y) with `score_l140.py` on the rebuilt ix
  set (w5-triangulation `ix_evidence.tsv`, confidence direct/derived/closed). Its ported dispatch
  (`bratio::dispatch`) sends 1,383 ix rows to L140 bup+bpser; 995 are bit-exact, 1,362 within 15 digits. The model
  (verify-r2-bratio analyze.out, statistics.md "Incomplete beta L140 and bfrac") counted 1,620 ix L140 cells, 1,011
  bit-exact. The 237-cell difference is not traced: candidates are the model's own dispatch (w5 `dispatch.py`,
  `bratio_excel.py` `kernel`), the closed forms taken first in the engine, and a different ix_evidence.tsv build.
- Data rebuild steps are in docs/research/t2506/RESULTS.md ("Data").

## Acceptance criteria
- [ ] The rows in the model's 1,620 and not in the engine's 1,383 (and the reverse) are listed by cause in a
      RESULTS.md under docs/research/t2893/.
- [ ] The engine's score on the model's 1,620 cells (or on the cells both call L140 bup+bpser) is recorded against
      the model's 1,011, and T-2506's criterion 2 is marked met or a kernel follow-up is filed with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving F, T, BINOM or NEGBINOM onto bratio (T-2847).
