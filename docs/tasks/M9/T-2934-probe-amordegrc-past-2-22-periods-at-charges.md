---
id: T-2934
title: "Probe AMORDEGRC past 2^22 periods at charges of 1E11 and more so the closed-form DEVIATION has evidence cells"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/financial.md "AMORDEGRC past 2^22 periods" (a DEVIATION, decision 0014) cites Excel cells where
the closed form still applies: a new probe case records Excel's AMORDEGRC at charges of 1E11 and more past 2^22
(4,194,304) periods, and the entry lists which of them RustSheet misses.

## Context
T-2918 raised `DEGRESSIVE_LOOP_PERIODS` (functions/financial/amortization.rs) from 2^20 to 2^22, so every cell of
probes/fin-amor-period-bound-c (1.1E6 to 2.4E6 periods) is now Excel's. Past 2^22 periods the kernel finishes the decay
in closed form, which drifts by about 1e-13 relative, so a charge of about 1E11 or more can be a unit off; no probe
has such a cell today (Excel-fidelity review of T-2918). Suggested cells, rate 1E-7 (a life of 1E7 periods, half
period near 1E7 - 3): `AMORDEGRC(1E20,39679,39813,300,P,1E-7,1)` and `AMORDEGRC(3E19,..)` for P = 4.5E6, 5E6, 6E6,
8E6; `AMORDEGRC(1E300,..,5E6,1E-7,1)`; a control below 1E11 (`AMORDEGRC(1E15,39679,39813,1,5E6,1E-7,1)`, ours
71626199). Follow the `excel-probe` skill (spec in eval/suites/probes, oracle fill on excel-graph).

## Acceptance criteria
- [ ] A probe case (for example probes/fin-amor-period-bound-d) with at least 8 cells as above has an excel-graph
      oracle sidecar.
- [ ] financial.md "AMORDEGRC past 2^22 periods" cites it under Evidence and names the cells ours misses; the misses
      stay known failures in eval/baseline (`make eval-baseline`, no regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the walk bound or the closed form (the bound is a work cap, T-2918).
