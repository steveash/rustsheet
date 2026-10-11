---
id: T-1274
title: "Case builder: store LAMBDA optional parameters ([y]) as Excel does"
milestone: M7
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A case spec with a LAMBDA optional parameter (`=LAMBDA(x,[y],ISOMITTED(y))(1)`) builds an xlsx that Excel for the web opens, so ISOMITTED and optional parameters can be probed.

## Context
T-1083: the case builder (tools/src/rustsheet_tools/spec2xlsx.py, `prefix_bound_names`) writes the bracketed parameter as `[_xlpm.y]`, and excel-graph refuses the workbook (`FileCorruptTryRepair`, "unsupported features"). Find how Excel stores an optional parameter (save one from Excel for the web through the oracle, or an Excel-authored file) and write that form. Then probe `LAMBDA(x,[y],ISOMITTED(y))(1)`, `(1,2)` and `LAMBDA(x,[y],IF(ISOMITTED(y),x,x+y))(1)` and record them (docs/excel-semantics/arrays.md, the M7 argument-modes entry, says the probe is pending on this).

## Acceptance criteria
- [ ] test_spec2xlsx covers the optional-parameter form; the three probes are filled by excel-graph and recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
