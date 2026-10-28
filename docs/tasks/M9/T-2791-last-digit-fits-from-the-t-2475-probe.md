---
id: T-2791
title: "Last-digit fits from the T-2475 probe: F.INV.RT(1-1E-10,8,7), NEGBINOMDIST(7,9,1E-10), SKEW(1E-15,1,2) (3 cells, split from T-2621)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
Three statistical cells that differ from Excel only in their last digits match bit for bit.

## Context
Split from T-2621 (the domain-error cells of probes/t2475-stat-subexpressions landed there; these three are numeric
fitting). Probe: probes/t2475-stat-subexpressions (excel-graph, 2026-10-24); known failures in eval/baseline/gen-m9.txt
and eval/baseline/probes.txt:
- gen-m9/873f30d2bf76f8db Calc!A308, probe P!A317 `F.INV.RT(1-1E-10,8,7)`: Excel 0.0012599666259138292, ours
  0.0012599666259138298 (fisher.rs F quantile; see statistics.md F.INV entries for the known kernel fits).
- probe P!A417 `NEGBINOMDIST(7,9,1E-10)`: Excel 6.434999995495541e-87, ours 6.434999995495502e-87 (binom.rs
  `negbinom_dist_of`); its gen-m9 owner a63d1c7d6f96c56d A92 already passes (the outer formula hides the digits).
- gen-m9/d208bbafc3bc1997 Calc!A262, probe P!A520 `SKEW(D41!D11:D12,D41!D10)` = SKEW over 1E-15, 1, 2: Excel
  2.4980018054066022e-15, ours 2.9976021664879227e-15 (count_and_average.rs `fn_skew`). T-2621 tried the obvious
  two-pass orders (sum of cubed deviations over s³, cubes of z, n/((n-1)(n-2)) placement, var^1.5): none is bit-exact;
  dividing the cube sum by s³ gives ...066066e-15, so the mean or the cube sum is formed differently in Excel. Probe
  more SKEW inputs (excel-probe skill) before fitting.

## Acceptance criteria
- [ ] The three probe cells and the two gen-m9 cells leave eval/baseline, no new known-failure lines; each fit is
      recorded in docs/excel-semantics/statistics.md with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other T-2475 follow-ups (T-2618 to T-2623).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the 1 visibly-off SKEW cell. Fixing them lifts SKEW to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.
