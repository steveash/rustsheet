---
id: T-1060
title: "Probe Excel acceptance of 8,192-character operator chains (+ and &)"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether Excel accepts the longest operator chains its 8,192-character formula
limit allows, and what they return, so the `MAX_VALUE_OPERATORS` (4,096) claim in
docs/excel-semantics/syntax.md ("Formula nesting depth cap") is evidenced, not derived.

## Context
T-0128 replaced the 255-term chain cap with a 4,096 value-operator cap, argued from the published
limit (`n` terms of `1+1+...` take `2n-1` characters). Nobody has entered such a formula in Excel.
Use the `excel-probe` skill; code: crates/rustsheet-engine/src/expressions/parser/depth.rs.

## Acceptance criteria
- [ ] A probe case enters `=1+1+...` and `=LEN(1&1&...)` at 8,191 and 8,192 formula characters (and one
  past the limit) and records Excel's values or refusal; RustSheet matches where Excel accepts.
- [ ] The syntax.md entry cites the probe instead of the arithmetic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_VALUE_OPERATORS` unless the evidence requires it.

## Outcome
Added probe `eval/suites/probes/cases/operator-chain-limit` (+ excel-graph oracle sidecar): `=1+1+...`
and `=LEN(1&1&...)` at 8,191, 8,192 and 8,193 formula characters. Excel for the web accepted and
evaluated all six (including the one past the limit, which came through a file load); RustSheet
matches all six, so `MAX_VALUE_OPERATORS` stays 4,096. syntax.md now cites the probe. Eval delta:
probes +1 case, +6 cells, all passing; no regressions.
