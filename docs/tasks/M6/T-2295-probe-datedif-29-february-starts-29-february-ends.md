---
id: T-2295
title: "Probe DATEDIF 29 February starts, 29 February ends in non-leap years, and negative MD"
milestone: M6
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values for the DATEDIF cases T-2264 left UNVERIFIED are known and recorded.

## Context
T-2264 implemented the "MD"/"YD" rule (docs/excel-semantics/dates.md "DATEDIF "MD" and "YD" day counts"). No oracle cell decides
three cases: a start on 29 February (`DATEDIF("2/29/2020","2/28/2021","YD")`, rule gives 365), an end of 29 February moved into a
non-leap year (we use 1 March), and a negative "MD" (2015-01-31 to 2015-03-01, rule gives -2). Use the `excel-probe` skill.
Code: `datedif_value` in crates/rustsheet-engine/src/functions/date_and_time.rs.

## Acceptance criteria
- [x] A probe case under eval/ covers each of the three cases with Excel-computed values.
- [x] dates.md's Status line is VERIFIED for them, or the code and its tests are fixed to the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Serials below 61 (T-2263).

## Outcome
Probe `probes/datedif-feb29-negative-md` (66 cells: 29 February starts and ends, negative "MD") filled from excel-graph; the
engine already matched all 66, so no code change. dates.md "DATEDIF "MD" and "YD" day counts" now records the three cases as
VERIFIED. Eval delta: +66 probe cells passing, no known-failure change.
