---
id: T-3220
title: "BETA.DIST density for min(a,b) > 1: the 661 cells the Loader port leaves visibly off (Loader constant and bd0 series residual)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2586]
campaign: visible
---

## Goal
BETA.DIST(x,a,b,FALSE) for min(a,b) > 1 is within 15 digits of Excel on the 661 probes-numeric cells T-2507's Loader
port leaves visibly off (docs/research/t2507/newly_visibly_off.tsv), and BETADIST_BETAINV.xlsx G3 is Excel's double
again.

## Context
- Decision 0181 let T-2507 merge with these cells: the port fixed 36,696 visibly-off BETA.DIST cells against `main`
  ec865788, and its shape constant (2-4 units) and bd0 series (3-41 units) carry the residuals T-2586 items 1 and 2
  are cracking. Each fix here is T-2586's model of those two pieces, ported into `loader.rs` `excel_beta_density`.
- Cells: every row of docs/research/t2507/newly_visibly_off.tsv (case stem and cell); plus ironcalc-calc
  STATISTICAL/BETADIST_BETAINV.xlsx Sheet1!G3, 1 ulp low since T-2507.

## Acceptance criteria
- [ ] The TSV's cells are within 15 digits (or each one left is named with the reason), with no new visibly-off cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- min(a,b) <= 1 (T-2503's M2, T-2821).

## Visible-parity stage
Target (decision 0114): every cell above within half a unit of the 15th significant digit of Excel's value. Bit-level
residue moves to a follow-up with `defer: phase-1b`.
