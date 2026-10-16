---
id: T-1660
title: "Score the spill extent of spec cases: cells absent from the oracle sidecar must be empty in the engine"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A spec case fails when the engine spills into a cell that Excel left empty, so a spill-extent divergence cannot hide.

## Context
The harness scores only the cells present in a case's oracle sidecar. In probes/ceiling-floor-mround-array-lift the engine spilled `{#VALUE!, 3}` for `MROUND(K1:L1,1)` where Excel gives one `#VALUE!` with B21 empty, and the case still scored 65/65 (found by the fidelity reviewer of T-1589). Decide how absence is recorded (an explicit empty entry in the oracle fill for the neighbours of a formula cell, or a per-case check of the engine's used area) and what it does to the existing baselines (expect new known failures to be recorded, never hand-edited).

## Acceptance criteria
- [x] A cell outside the oracle's results that the engine fills is reported as a mismatch for every spec suite; the baselines are regenerated with `make eval-baseline`, with the new known failures listed in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Every spec case with an Excel sidecar now gets a spill-extent check (`SpillExtent`, runner.rs). Each cell the engine stores that is neither an input nor recorded by the oracle fails against empty. Excepted are cells that fill the rectangle below and to the right of a formula the oracle left pending, since Excel's extent there is unknown. The check reads the engine's stored cells through a new read-only `Workbook::stored_cells`, which is sparse, so far-corner inputs cost nothing. Extent cells stay out of `checked`, so a later engine change that spills further is a regression (decision 0063). The existing mismatches were recorded once with `--allow-regressions` under that decision:
- probes/eta-reduced-names Sheet1!E50, E51, E102, E103: BYROW whose function returns an array is one `#CALC!` in Excel, but the engine spills. Follow-up T-1664.
- probes/radians-sqrtpi-munit Sheet1!B80: `SQRTPI(D1:E1)` is one `#VALUE!` in Excel, but the engine spills. Follow-up T-1663.

probes/ceiling-floor-mround-array-lift (the case that prompted this task) already agrees on main. The ungated suites also show extent mismatches. gen-m7/93976dd9a3578ed9 Calc!B10:C10 is `#NAME?` because GROUPBY is unimplemented (T-1322). gen-m9/b5352ebdb9c5f1a1 Calc!A41 comes from `ISBLANK(LEN(BYROW(..,LAMBDA(v,v))))`, which is the same one-cell BYROW `#CALC!` (added to T-1664).

Eval: probes known failures 5563 → 5568. All 5 are extent cells; no cell that passed before now fails.
