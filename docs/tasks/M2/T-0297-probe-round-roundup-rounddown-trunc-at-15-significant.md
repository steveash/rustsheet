---
id: T-0297
title: "Probe ROUND/ROUNDUP/ROUNDDOWN/TRUNC at 15 significant digits and huge digit counts (ironcalc b93d4224)"
milestone: M2
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ROUND, ROUNDUP, ROUNDDOWN and TRUNC match excel-graph at rounding boundaries reached through binary noise and at digit counts far outside f64's range.

## Context
Upstream IronCalc b93d4224 (2026-09-30, first listed 2026-10-09) made the four functions round at 15
significant digits first, so ROUND(1.005,2)=1.01 and TRUNC(-(0.1+0.7),1)=-0.8, and changed huge digit
counts: ROUND(1E+307*10,10) keeps the value (was #NUM!), ROUND(123.456,-400) and TRUNC(123.456,-16)
are 0 (were #NUM! and 123.456). Upstream is not evidence: probe each of these and the ROUNDUP/ROUNDDOWN
analogues with the excel-probe skill (existing coverage: `probes/precision-and-rounding` has
ROUND(1.005,2)), compare with our `functions/math_and_trigonometry` implementation, and port only
what excel-graph confirms (`Ported-From: ironcalc@b93d42242b64fc7a356e84d70c7c1e7b1d7382f0` if code is
taken). Record each rule in docs/excel-semantics/.

## Acceptance criteria
- [ ] A probe case with the upstream examples (and ROUNDUP/ROUNDDOWN, negative digits, digits beyond +-308) is oracle-filled.
- [ ] The engine matches every cell, or the remaining ones have filed tasks; findings recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MROUND, FLOOR, CEILING.
