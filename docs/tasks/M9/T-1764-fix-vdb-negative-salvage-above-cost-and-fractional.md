---
id: T-1764
title: "Fix VDB negative-salvage-above-cost and fractional end-period, AMORDEGRC rate thresholds (probes fin-vdb-modes, fin-amordegrc)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VDB and AMORDEGRC match Excel on the cells the T-1625 probes found failing.

## Context
Also the last-ulp differences in `probes/fin-mduration` (A1-A6, A8, A17) and `probes/fin-rri` A14.

Probes `probes/fin-vdb-modes` (A6, A7, A9, A10, A13 ulp/close; A9 `VDB(2400,300,10,9.5,10,2,FALSE)` Excel 0 vs ours 11.06; A17 `VDB(2400,3000,10,0,1)` Excel -600 vs ours -60) and `probes/fin-amordegrc` (A10, A11, A13, A14: rate 0.20 / 0.25 with a ~1.5 year life gives numbers in Excel, we give #NUM!). See docs/excel-semantics/financial.md (T-1625 entry).

## Acceptance criteria
- [ ] every failing cell of those two probe cases passes or is documented as a DEVIATION; entries leave eval/baseline/probes.txt
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
VDB with salvage above cost (A17) and a fractional end period at the end of life (A9) now match Excel, and
AMORDEGRC accepts a life of exactly 4 and 5 (A10, A11, A13, A14); probes/fin-amordegrc passes fully. The
last-ulp cells (VDB A6, A7, A10, A13, MDURATION, RRI) need numeric fitting and moved to T-1975 (opus).
