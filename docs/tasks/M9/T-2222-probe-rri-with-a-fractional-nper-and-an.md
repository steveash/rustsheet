---
id: T-2222
title: "Probe RRI with a fractional nper and an inexact ratio (Mac desktop RRI(0.5,3,4) is 0.7777777777777775, the worksheet form ...777)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel for the web's value of `RRI(0.5,3,4)` and nearby cells is in the repo. Then either the engine
is changed to match, or the ironcalc-calc cell is shown to be a Mac-desktop-only difference and left
to M14.

## Context
- ironcalc-calc/FINANCIAL/RRI.xlsx Sheet1!D12 `=_xlfn.RRI(A12,B12,C12)` with A12 = 0.5, B12 = 3,
  C12 = 4 caches 0.7777777777777775. That is a Mac desktop value: "Microsoft Macintosh Excel" 16.0300.
  The engine gives 0.7777777777777777. It is the only failing cell of that workbook (a known-failure
  line in eval/baseline/ironcalc-calc.txt) and no task owns it (gap triage 2026-10-21).
- docs/excel-semantics/financial.md "DURATION and RRI operation order" (VERIFIED, excel-graph,
  T-1975) says RRI is exactly the worksheet `(fv/pv)^(1/nper)-1`. Here that is `(4/3)^2-1`, which the
  engine gives. The web probes behind that rule (fin-rri, fin-mduration-rri-fit, fin-rri-vdb-edges) have
  integer nper or an exact ratio. None has a fractional nper with an inexact `fv/pv`, so the Mac cell
  neither confirms nor refutes the rule.
- The same shape of question was settled for SIN(4) and ATAN2 (Mac desktop values, decisions 0074 and
  0085), where the web agreed with the engine.

## Acceptance criteria
- [ ] A probes case `eval/suites/probes/cases/rri-fractional-nper.case.json`, filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes` (excel-probe skill), has
      `RRI(0.5,3,4)` and, beside each RRI, the worksheet form `(fv/pv)^(1/nper)-1`. It covers:
      nper in {0.5, 0.25, 1.5, 2.5, 1/3}, (pv, fv) in {(3,4), (3,5), (7,10), (1,3), (10,7)}, and
      `POWER(4/3,2)-1` and `(4/3)*(4/3)-1`. The case carries exactly one milestone tag, `M9`
      (decision 0057, scripts/check_governance.py).
- [ ] financial.md "DURATION and RRI operation order" records the result with the cells as evidence:
      - If the web matches the worksheet form, the RRI.xlsx D12 difference is recorded as
        Mac-desktop-only, and an M14 item is added to T-1075 (decision 0039).
      - If the web matches 0.7777777777777775, a follow-up functions-financial task is filed with the
        cells.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `rri_of` (functions/financial/annuity.rs) in this task.
