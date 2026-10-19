---
id: T-2029
title: "Generate a per-function Excel parity page (docs/parity.md) with SVG charts from the eval report"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A page on GitHub the owner can link people to shows RustSheet's per-function Excel parity: how many
in-scope functions are implemented, and how many are bit-exact, within 1 ulp, within 4 ulp,
within 1e-9 relative, or off, with charts. It is generated from the eval report and refreshed by
one command.

## Context
The numbers come from `make eval`'s `eval/runs/latest/report.json`: per-function pass and cell
counts from its `functions` list (deviations pass, unsupported cells are not scored), and the
failing cells from `cases[].cells`, attributed to functions as `functions_in` in
crates/rustsheet-eval/src/report.rs does. `numeric_ulp` cells split at 1 ulp with the ordered-bits
distance of `ulp_distance` in crates/rustsheet-eval/src/value.rs. Only catalog functions with
`scope: in` (eval/catalog/scope.json) count. A one-off script produced the numbers the owner has
seen; this task turns it into a typed, tested generator. The Makefile is protected, so the target
comes with decision 0079.

## Acceptance criteria
- [x] `tools/src/rustsheet_tools/parity_report.py` (entry point `rustsheet-parity-report`, target
  `make parity-report`) writes docs/parity.md, docs/parity/functions.md and three SVG charts
  (functions per bucket, functions per category stacked by bucket, checked cells per bucket).
- [x] It fails loudly when a function's buckets do not sum to its scored cells.
- [x] On the report of main c30e2d40 (2026-10-19): 505 functions; pass 238, ulp1 20, ulp4 17,
  close 32, off 198; cells 258,865 / 2,151 / 1,532 / 5,273 / 2,730.
- [x] The SVGs carry a `prefers-color-scheme: dark` style and an explicit fill on every shape.
- [x] tools/tests/test_parity_report.py covers the ulp split, the worst-bucket rule, deviation as
  pass, unsupported dropped and multi-function attribution.
- [x] docs/README.md links docs/parity.md; `make hygiene`, `make typos` and `make py-check` green.

## Out of scope
- Publishing the page anywhere but the repository, and running the generator in CI or a routine.
- Changing what the eval report scores.

## Outcome
Generator in tools/src/rustsheet_tools/parity_report.py (charts in parity_svg.py), tests in
tools/tests/test_parity_report.py, target `make parity-report` (decision 0079). Generated from the
2026-10-19 report on c30e2d40: 505 of 506 in-scope functions scored (INFO has no scored cell);
238 bit-exact, 20 within 1 ulp, 17 within 4 ulp, 32 within 1e-9, 198 off (45 of them still
bit-exact on 99% or more of their cells); 270,551 function-cells, 95.68% bit-exact. Matches the
one-off script's per-function output exactly. Follow-up idea, not filed: the steward could run
`make parity-report` after its daily `make eval`.
