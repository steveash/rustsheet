---
id: T-1453
title: "ISFORMULA of INDEX(range,n) is #VALUE!: INDEX is not a reference in get_reference context (probes/ifna-isformula-edges Calc!B12)"
milestone: M5
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`ISFORMULA(INDEX(Data!D1:D3,1))` is TRUE (a reference-returning INDEX is accepted wherever a reference argument is), as Excel computes it.

## Context
Excel (excel-graph) gives TRUE for probes/ifna-isformula-edges Calc!B12; the engine gives #VALUE! ("Expected reference") because `get_reference` (crates/rustsheet-engine/src/cast.rs) uses `evaluate_node_with_reference` (model.rs), which does not yield a Range for an INDEX call. OFFSET and INDIRECT already work (B10, B11 pass). Check other reference-taking functions (ROWS, COLUMNS, SHEET) for the same gap; ISREF(INDEX(..)) already works (T-0216) and COUNTA treats INDEX as a picked reference (T-1395).

## Acceptance criteria
- [ ] Calc!B12 of probes/ifna-isformula-edges passes and leaves the probes baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
