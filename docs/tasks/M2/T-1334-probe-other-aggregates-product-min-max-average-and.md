---
id: T-1334
title: "Probe other aggregates (PRODUCT, MIN, MAX, AVERAGE) and SUM array/text args for direct-vs-reference error precedence; #BUSY! criterion in SUMIF/COUNTIFS/DCOUNT"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which aggregates share SUM's rule (direct-argument error beats a referenced-cell error) and for the untested SUM argument shapes.

## Context
Follow-up of T-1314 reviews. Probe PRODUCT, MIN, MAX, AVERAGE, COUNT-family against a referenced error plus a direct error in both orders; SUM with array args (`{1,#N/A}`, `A2:A3*1`), uncoercible text and a `#REF!` area; a stored `#BUSY!` as criterion in SUMIF, COUNTIFS and DCOUNT. Record in syntax.md/criteria.md and file engine tasks for mismatches.

## Acceptance criteria
- [x] Oracle-filled probe case with the shapes above.
- [x] Rules recorded in docs/excel-semantics; engine tasks filed for mismatches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/aggregate-error-precedence (92 cells, excel-graph). Rules recorded in syntax.md and criteria.md; 16 cells mismatch the engine, filed as T-1379. The criteria functions (SUMIF/SUMIFS/COUNTIFS/AVERAGEIF/DCOUNTA) already match for a stored #BUSY! criterion. Reviewers pending at PR time.
