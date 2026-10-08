---
id: T-1005
title: "Oracle sweep for T-0917: inverse hyperbolics and SINH/TANH switchover thresholds"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T-0917 can fit Excel's ASINH/ACOSH/ATANH/ACOTH/COTH/CSCH and SINH/TANH formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-hyperbolic.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: ASINH/ACOSH/ATANH/ACOTH on log-spaced arguments (1e-10..1e10, and near 1 for ACOSH/ATANH/ACOTH: 1±1e-k), COTH/CSCH on 1e-10..710, and SINH/TANH on a fine grid around their switchover thresholds (bisect where results stop matching libm; T-0914 notes the fitted formulation).

## Acceptance criteria
- [ ] `m9-hyperbolic.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-hyperbolic.case.json`) are recorded in docs/excel-semantics/numbers.md with the case as evidence.
- [ ] T-0917 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0917 does that).

## Outcome
Added `probes-numeric/m9-hyperbolic` (268 cells) with its excel-graph oracle sidecar (no pending cells). 110 of 268 cells differ from the engine (66 ulp, 31 close, 11 mismatch, 2 SINH(+-710) `#NUM!`); quirks recorded in numbers.md. T-0917 already depends on T-1005 and no longer on T-0942. Eval baseline unchanged (probe suite carries its own known failures; see make check).

## Handoff (resumed)
Tried: built `probes-numeric/m9-hyperbolic` (268 cells) and filled it from excel-graph; `make check` passed; security, architecture and quality reviewers approved. The fidelity reviewer requested changes twice, both times on the numbers.md section "oracle sweep (T-1005)" (the case and oracle data themselves were confirmed sound).
Round 1 was fixed (ASINH/ACOSH are the plain log formulas, 65/65 bit-for-bit; near-1 ACOTH/ATANH is not a quirk; COTH/CSCH wording; SINH(709.78) was probed). Round 2 is open: the line "ACOTH for large x (A101-A110) equals libm `atanh(1/x)` bit-for-bit" is false. Only A105 and A107-A110 match bit-for-bit; A101, A102, A104 and A106 are 1 ulp off glibc and Rust `atanh` (A103 too for glibc). Excel is not correctly rounded there either.
Next: reword that bullet to "tracks atanh(1/x) within 1 ulp; bit-for-bit on A105, A107-A110; formula left open for T-0917", re-run `reviewer-excel-fidelity`, then set `status: done` (keep the Outcome section below), merge on green CI. Also note the Outcome says "tier: sonnet" work was done; the case is not to be regenerated (duplicates are harmless).
