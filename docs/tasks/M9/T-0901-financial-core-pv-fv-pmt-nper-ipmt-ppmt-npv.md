---
id: T-0901
title: "Financial core (PV/FV/PMT/NPER/IPMT/PPMT/NPV) bit-exact parity"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0942]
---

## Goal
Implement the fitted annuity formulations: PV and FV (already 308/308 on the sweep), NPER (95/95) and
an accurate double-precision PMT (all of m9-fin-pmt within 4 ulp). The PMT, IPMT, PPMT and NPV last
digits that no formulation reproduces yet are T-1528 (time-boxed).

## Context
Kernel map (T-1522, decision 0056): rank 1 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Gap report: PV 47, NPV 40, PPMT 34, PMT 26, NPER 26, IPMT 21, FV 20 failing cells.

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-0942 sweep, never blind: `eval/suites/probes-numeric/cases/`
`m9-fin-power`, `m9-fin-pv-fv`, `m9-fin-pmt`, `m9-fin-nper-npv` (oracle-filled 2026-10-06). The PV and FV
formulations already fit 308/308 each (docs/excel-semantics/financial.md); PMT does not yet.

**T-1524 (fit lab `annuity-*`, 2026-10-14):** NPER is fitted: `ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)`
in doubles (k = 1+r*type; each logarithm a double logarithm of the double 1+r, libm or correctly
rounded, which the sweep does not separate) matches 95/95 numeric cells of m9-fin-nper-npv (engine 61);
`log1p(r)` gives 11. PV and FV refit 308/308 in the lab. Excel's annuity code is plain double
arithmetic: keeping intermediates at 64 bits (x87 registers) drops PV to 174, FV to 230, NPER to 6.
PMT stays unexplained. Every model accurate to about half an ulp (correctly rounded, 64-bit intermediates
on the exact or fyl2x/f2xm1/fscale power) gets 132/308, because Excel's own PMT lies up to 3.5 ulps from
the exact value. An implementable gain: PMT in doubles with `P-1 = expm1(n*ln_1p(r))` and
`-(pv*P+fv)/((P-1)/r*k)` gives 120 bit-exact (engine 54) and 308/308 within 4 ulp (engine 176), and
`-(pv*r+(pv+fv)*r/(P-1))/k` gives 129 (306 within 4 ulp). The engine's `(1+r)^n - 1` cancels for small r*n.
IPMT/PPMT best 219/94 (correctly rounded), NPV best 27/88: moved to T-1528 (time-boxed, decision 0056)
together with PMT's exact last digits. Details:
docs/excel-semantics/financial.md "Annuity evaluation order"; numbers.md "x87 precision is per operator, not
per kernel".

## Acceptance criteria
- [x] NPER computes `ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)` in doubles (financial.md): all 95 numeric
      NPER cells of `probes-numeric/m9-fin-nper-npv` pass, and no new lines in any `eval/baseline/` file.
- [x] PV and FV use financial.md's fitted formulation (P by Excel's square-and-multiply, `k*((P-1)/r)`):
      all 308 PV and 308 FV cells of `probes-numeric/m9-fin-pv-fv` pass, no new baseline lines.
- [ ] (moved to T-1528, see Outcome) PMT uses `P-1 = expm1(n*ln_1p(r))` (no `(1+r)^n - 1` cancellation), for example
      `-(pv*P+fv)/((P-1)/r*k)`: on `probes-numeric/m9-fin-pmt` at least 120 of 308 PMT cells bit-exact and
      308/308 within 4 ulp (today 54 and 176), no new baseline lines anywhere (IPMT/PPMT, which call it,
      included).
- [x] financial.md's "Implemented in" names the code; `make check` green; reviewers approve.

## Out of scope
IRR/XIRR/RATE solvers (separate task).

## Earlier attempt (draft PR #13, 2026-10-02; salvaged by T-0031)
Blocked then for lack of an oracle; the oracle now exists (decision 0013), so this is ready. Fitting
to cached values alone gained ~39 cells but regressed 13 (details in docs/excel-semantics/financial.md).
Probe PV/FV/PMT on integer-`nper` grids with the oracle first (excel-probe skill), then re-fit.

## Outcome
2026-10-16. PV, FV and NPER use financial.md's fitted formulations (`annuity_power`,
`annuity_factor`, `FutureValueForm::Fitted`), with oracle values pinned in
test/test_fn_annuity_fits.rs:
- **Sweeps** (probes-numeric, not gated): PV 308/308, FV 308/308 (engine before: 107 and 101 of
  the numeric cells), NPER 132/132. NPER's last division is Excel's twice-rounded `/`: plain
  division misses A9.
- **Gated suites:** ironcalc-calc fixed 35, poi fixed 8, probes fixed 1. Eight PV/NPER cells of
  PMT.xlsx, fed by PMT cells that already fail, were unmasked and are allowed by decision 0064.
  They join T-1528.
- **Not done here:** the accurate PMT (expm1 form) and IPMT on the fitted FV. The PMT form gives
  120/308 on the sweep, but on the Excel-cached workbooks it fixes 30 cells and regresses 22.
  IPMT's recurrence on the fitted FV regresses 4. Excel's PMT is not the accurate value, so both
  stay with T-1528, and IPMT keeps the old recurrence (`FutureValueForm::IpmtRecurrence`).
