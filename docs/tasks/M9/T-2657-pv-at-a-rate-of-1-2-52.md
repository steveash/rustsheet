---
id: T-2657
title: "PV at a rate of -1+2^-52 and IRR from a guess of 42 are #DIV/0! (gen-m9, 2 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PV and IRR give Excel's `#DIV/0!` where the rate makes a growth factor underflow or the iteration break down:
`PV(-1+2^-52,91,-421)` and `IRR(<flows>,41.9999)` over the cells below.

## Context
- T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, excel-graph, 2026-10-25), 2 gen-m9
  cells, all visibly off; the enclosing financial call passes the inner error on:

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/c0681a8b150ce301 Calc!A101 | `PV((-1+2^-52),91,-421)` (inside MDURATION) | #DIV/0! | #NUM! |
| gen-m9/9d9ceba257f8001b Calc!A288 | `IRR(Data!A6:D9,(Data!$D$6+(-0.0001)))` (guess 41.9999; inside PRICEDISC) | #DIV/0! | 4101195592516.4434 |

- PV: (1+rate)^91 = (2^-52)^91 underflows to 0, and Excel's `#DIV/0!` fits a division by that factor; ours reports
  `#NUM!`. IRR: from a guess of 41.9999 Excel stops with `#DIV/0!` where ours
  converges on 4.1E+12. T-2493 (NPV at a rate of -1 or 1E+300 is `#DIV/0!`) and T-1686 (IRR caps and overflowing trial rates) are
  the neighbours: reuse their findings and probes.
- Recorded in docs/excel-semantics/financial.md "Financial cells of the T-2496 sub-expression probe" (one cell each; the
  boundaries are not probed).

## Acceptance criteria
- [ ] An excel-graph probe (`excel-probe` skill) of PV, FV and PMT with rate -1+k*2^-52 and n from 1 to 200, plus IRR over the cell's flows from guesses 1, 10, 41.9999 and 1E+3, pins where `#DIV/0!` starts.
- [ ] The engine matches the probe.
- [ ] Visible parity (gen-m9, T-2496): the 2 known-failure lines above disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A financial.md entry of its own for PV and IRR, with the probe's boundaries, replaces their sentence in that section.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NPV at a rate of -1 or 1E+300 (T-2493) and the IRR iterate's last digits (T-2131).
