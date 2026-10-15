---
id: T-1573
title: "MAXA, MINA and AVERAGEA rank a scalar direct error above a referenced one, leftmost referenced error first"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MAXA, MINA and AVERAGEA return a scalar direct error before a referenced error, and the leftmost referenced error otherwise, like MAX/MIN/AVERAGE.

## Context
probes/aggregate-picked-direct-a-variants D20-D27 (ours: first error at once, `MAXA(A5,#DIV/0!)` is #N/A instead of #DIV/0!). `for_each_value_a`; SPEC-0009 row AVERAGEA, MAXA, MINA.

## Acceptance criteria
- [x] The probe cells named in Context pass (`rustsheet-eval show`), no new known failures; the SPEC-0009 row is updated from DIVERGES to matched.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cases the probe did not cover.

## Outcome
AVERAGEA, MAXA and MINA use the AVERAGE family's error policy (`Ranked`, direct array errors): a scalar direct error beats a referenced one and the leftmost referenced error wins. probes/aggregate-picked-direct-a-variants D20 and D22 now pass (D21, D23-D27 already did); probes known failures 5579 -> 5577, no regressions.
