---
id: T-1601
title: "Probe PRODUCT over TRIMRANGE and over a picked error cell with a later direct error"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for PRODUCT over a TRIMRANGE pick and over a picked error cell followed by a direct error.

## Context
T-1571 made PRODUCT read picks like SUM (evaluate_aggregate_arg); these two cases follow SUM's rule without PRODUCT oracle values (docs/excel-semantics/syntax.md, UNVERIFIED). Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers both and a picked boolean/blank cell; SPEC-0009 PRODUCT row and syntax.md updated, and ours fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/aggregate-product-trimrange-picks` (15 cells, excel-graph): PRODUCT matches SUM for TRIMRANGE picks, picked boolean/blank cells and picked-error ranking; ours already agrees, so no code change. SPEC-0009 and syntax.md updated. Eval: new probe passes 15/15; the baseline registers the new case (one generated C line), no F lines.
