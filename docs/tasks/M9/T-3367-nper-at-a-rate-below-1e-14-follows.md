---
id: T-3367
title: "NPER at a positive rate below 1E-14 follows the math, not Excel's up-to-4% error (decision 0188 R3)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3355]
---

## Goal
NPER at 0 < rate < 1e-14 (excluding rates where 1 + rate == 1,
which stay `#DIV/0!`) returns the math (the zero-rate limit and its first-order correction) instead of reproducing
Excel's result, which is up to 4% off there (decision 0188 R3, `MATH-OVER-EXCEL`).

## Context
- Owner's 2026-10-31 assessment (decision 0188, "Where Excel itself is far from the math"): NPER at rates below 1e-14 is up
  to 4% off in Excel, above G = 1e-4, so the engine follows the math there; financial.md's `MATH-OVER-EXCEL` entry
  records it. Today the engine matches Excel bit for bit.
- Code: `fn_nper` in crates/rustsheet-engine/src/functions/financial/annuity.rs.

## Acceptance criteria
- [ ] A probe case (one milestone tag, M9), filled by the oracle, holds NPER points at rates from 1e-17 to 1e-12 with their
      50-digit math values; the financial.md entry cites it and states the region where Excel's error exceeds 1e-4.
- [ ] In that region the engine is within 1e-9 relative of the math; outside it the engine still matches Excel (within
      15 digits).
- [ ] Every cell where Excel returns an error (`#DIV/0!` where 1 + rate == 1, `#NUM!`) keeps Excel's error.
- [ ] Negative tiny rates are out of the region; if they are to be covered, probe them first (a separate acceptance in
      a follow-up task).
- [ ] Cells that leave Excel's value are listed in the Outcome; no other gated cell regresses.
- [ ] Before any engine change follows the math, the region's evidence points (oracle-filled, with 50-digit math
      values) and its bounds are committed and the `MATH-OVER-EXCEL` entry cites them; the engine change comes after,
      in the same PR or a later one.
- [ ] The math path keeps directive 5's caps (bounded iterations, no panic); decision 0188 R3 never overrides them.
- [ ] R3 cells are marked as the Gate A spec (T-3355) says. Interim route until T-3359 lands: the cells that leave
      Excel's value are listed in the Outcome, they still count as visibly off in the report, and their baseline
      regression is recorded with a decision record citing decision 0188 R3.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other financial functions.
