---
id: T-0042
title: "spec2xlsx: write spill references A1# as _xlfn.ANCHORARRAY(A1)"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Spec cases can use spill references (`A1#`) and Excel for the web loads them.

## Context
spec2xlsx writes formula text into `<f>` unchanged. A file stores a spill reference as
`_xlfn.ANCHORARRAY(A1)`, and Excel for the web refuses a workbook whose formula holds a literal
`J1#` (`FileCorruptTryRepair`; T-0138 scratch run, 2026-10-05). The engine reads the spec text,
so cases keep `A1#`. Only the xlsx sent to the oracle needs the file form.
probes/intersection-operands H12 uses `=ANCHORARRAY(J1) J2:K2` for this reason, and the engine
does not know the ANCHORARRAY function (`#NAME?`).

## Acceptance criteria
- [ ] `build_xlsx` writes `A1#`, `$A$1#` and `Sheet!A1#` as `_xlfn.ANCHORARRAY(...)` outside
  string literals; unit test in tools/.
- [ ] probes/intersection-operands H12 is `=J1# J2:K2`, its sidecar is deleted and
  refilled (`rustsheet-oracle fill`; `--force` would refill every case), and the cell passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An ANCHORARRAY function in the engine.
