---
id: T-0986
title: "TEXT General: drop one digit (6 to 5) when decimal exponent magnitude is 99 or more"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT(x,"General") prints 5 significant digits when the decimal exponent magnitude is 99 or more (6 otherwise), as Excel does.

## Context
Rule in docs/excel-semantics/numbers.md (Exponent width); evidence probes/number-text-exponent-digits column D (T-0186). Engine fails D1-D8 and D10-D15.

## Acceptance criteria
- [x] number-text-exponent-digits column D cells pass (except D8, see Outcome); baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The formatter's General branch prints 4 decimals (5 significant digits) when the decimal exponent magnitude is 99 or more. Probes known failures 277 -> 264 (fixed 13: D1-D7, D10-D15), no regressions. D8 (`1.23456789012345E-5`, E notation at a small exponent) is not an exponent-width case: it needs the 11-character fixed-versus-E rule of T-0938 and stays in the baseline.
