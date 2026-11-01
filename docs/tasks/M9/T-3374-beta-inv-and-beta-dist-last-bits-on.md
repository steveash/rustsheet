---
id: T-3374
title: "BETA.INV and BETA.DIST last bits on BETADIST_BETAINV (H23, H32, H42-H49, G65; 1-4 ulp, from T-2633)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2511]
defer: phase-1b
---

## Goal
The BETA.INV and BETA.DIST cells of `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` that are within 4 ulp of Excel
but not bit-exact match Excel to the bit, and their lines leave `eval/baseline/ironcalc-calc.txt`.

## Context
- From T-2633 (its Stage: "cells left within 4 ulp ... go to a `defer: phase-1b` follow-up"). Measured on `main`
  95513f6 (after T-2510), `Sheet1`, Excel against ours:
  - H23 `BETA.INV(G23,..)`: -0.23399999999999987 vs -0.23399999999999976 (4 ulp);
  - H42, H44, H46, H47, H48, H49 `BETA.INV(G4x,..)`: 0.23399999999999999 vs 0.23400000000000004 (1 ulp; G42 itself is
    bit-exact now, so the residue is BETA.INV's own inversion, not its input);
  - H32 `BETA.INV(G32,..)`: 0.555 vs 0.5550000000000002 (added by T-2506);
  - G65 `BETA.DIST(..)`: 0.39911862276339233 vs 0.3991186227633924 (1 ulp; added by T-2506).
- The BETA.INV lines depend on how the inverse is solved to the crossing of BETA.DIST's kernel (T-2511, SPEC-0025
  E8): re-measure them on T-2511's inverse first; whatever it fixes leaves this task.
- Code: `functions/statistical/inverse_beta.rs`, `special::refine_increasing`, `bratio::beta_dist_lower_tail`.
  Semantics: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)".

## Acceptance criteria
- [ ] Each cell above is bit-exact, or its residue is recorded with the model tried in statistics.md.
- [ ] Their lines leave `eval/baseline/ironcalc-calc.txt`, with no new known-failure line in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and G26 (the T-1044 sweep).

## Deferred (decision 0114)
`defer: phase-1b`: every cell here is within 15 significant digits of Excel, so it waits until Phase 1 exits.
