---
id: T-0172
title: "xlsx export writes classic CEILING as _xlfn.CEILING (check against Excel)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The exporter writes the classic two-argument CEILING with the correct file form, which Excel stores without a prefix.

## Context
`Function::to_xlsx_string` (crates/rustsheet-engine/src/functions/mod.rs) maps `Function::Ceiling` to `_xlfn.CEILING`, but CEILING is a classic function (CEILING.MATH/PRECISE are the prefixed ones). T-0166 had to exclude it when reusing the list. Confirm with an oracle probe (excel-probe skill), fix, and drop the exclusion in tools/src/rustsheet_tools/spec2xlsx.py.

## Acceptance criteria
- [ ] Oracle evidence recorded in docs/excel-semantics; exporter and import round-trip fixed; spec2xlsx exclusion removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe ceiling-xlfn-prefix (excel-graph): Excel stores classic CEILING unprefixed and also loads
`_xlfn.CEILING` as CEILING. Exporter no longer prefixes it; spec2xlsx exclusion removed. Eval delta:
probes gains one case (4 cells, all pass); no known-failure change.
