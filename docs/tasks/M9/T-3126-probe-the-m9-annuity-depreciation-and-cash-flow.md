---
id: T-3126
title: "Probe the M9 annuity, depreciation and cash-flow argument kinds no Excel cell covers yet: empty text, an error or numeric text in 28 functions"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M9 annuity, depreciation and cash-flow functions below (PV/FV/PMT/NPER/RATE family, IPMT/PPMT/ISPMT, CUMIPMT/CUMPRINC, DB/DDB/SLN/SYD/VDB, AMORLINC/AMORDEGRC, EFFECT/NOMINAL, DOLLARDE/DOLLARFR, NPV/MIRR/RRI/PDURATION, XIRR/XNPV guess and rate) has at least one Excel-backed cell for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-29 (`make eval` on 6793e919 with calipers, corpus-bench and poi fetched; report.json
`argument_coverage`, decision 0031 evidence gaps; decision 0094 asks for each argument mode to be covered). Every
visibly-off cell of that run has an open owner (`scripts/exit_owners.py --all-off`), and the evidence floor is met, so
the M9 argument kinds are the largest evidence gap left: 245 M9 functions have a value parameter with a kind no Excel
cell passes (M2 was closed by T-2861; M5 to M7 are T-2862, T-2863, T-2864, T-2865 and T-2871 to T-2878; M9 had none).
The kinds are those of `crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative,
fraction, numeric_text, text, empty_text, boolean, error, blank, range. A kind is seen only when an Excel-evidence cell
passes it as a literal or a single-cell reference: nested calls such as `NA()` are not classified (docs/progress.md
"Argument coverage"), so use literal errors (`#N/A`, `#DIV/0!`) and cells holding them. T-2866 taught the generator to
draw `""` and error literals, but at 3% per parameter the existing gen-m9 cases predate it and do not reach these pairs.
`array`-mode parameters are left out: their coverage is not classified by kind.
financial.md "Financial function argument modes" records the probes/fin-argument-modes evidence these extend (it has no empty text or literal error). Choose valid other arguments away from the solvers' and payment formulas' edge cases, so a coercion difference is not confused with a last-digit one (financial.md "PMT, IPMT and PPMT are the exact payment").
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- PMT.rate: empty text; PMT.nper: empty text; PMT.pv: empty text; PMT.fv: empty text; PMT.type: empty text, an error.
- NPV.rate: empty text.
- FV.rate: empty text; FV.nper: empty text; FV.pmt: numeric text, empty text; FV.pv: numeric text, empty text, a boolean; FV.type: numeric text, empty text, an error.
- PV.rate: numeric text, empty text; PV.nper: numeric text, empty text; PV.pmt: zero, numeric text, empty text; PV.fv: numeric text, empty text; PV.type: empty text, an error.
- NPER.rate: empty text; NPER.pmt: empty text; NPER.pv: empty text, a boolean; NPER.fv: empty text; NPER.type: empty text, an error.
- RATE.nper: empty text; RATE.pmt: numeric text, empty text, a boolean; RATE.pv: numeric text, empty text; RATE.fv: numeric text, text, empty text, a boolean, an error; RATE.type: empty text, an error; RATE.guess: an error.
- XIRR.guess: numeric text, an error.
- XNPV.rate: empty text, an error.
- IPMT.rate: empty text; IPMT.per: numeric text, empty text, an error; IPMT.nper: numeric text, empty text, a boolean, an error; IPMT.pv: numeric text, empty text, an error; IPMT.fv: numeric text, empty text, an error; IPMT.type: empty text, an error.
- PPMT.rate: empty text; PPMT.per: empty text, an error, a blank cell; PPMT.nper: empty text, an error; PPMT.pv: empty text, an error; PPMT.fv: numeric text, empty text, a boolean, an error; PPMT.type: empty text, an error.
- SLN.cost: numeric text, empty text; SLN.salvage: empty text, an error; SLN.life: empty text, a boolean, an error.
- DB.cost: numeric text, empty text, an error; DB.salvage: numeric text, empty text, an error; DB.life: numeric text, empty text, an error; DB.period: empty text, an error; DB.month: empty text, an error.
- DDB.cost: empty text, an error; DDB.salvage: numeric text, empty text, an error; DDB.life: numeric text, empty text, an error; DDB.period: empty text, an error; DDB.factor: empty text, an error.
- CUMIPMT.rate: numeric text, empty text; CUMIPMT.nper: numeric text, empty text, a boolean, an error, a blank cell; CUMIPMT.pv: empty text, an error; CUMIPMT.start_period: numeric text, empty text, an error; CUMIPMT.end_period: empty text, an error; CUMIPMT.type: empty text, an error.
- CUMPRINC.rate: empty text; CUMPRINC.nper: numeric text, empty text, an error; CUMPRINC.pv: empty text, an error; CUMPRINC.start_period: empty text, an error; CUMPRINC.end_period: empty text, an error; CUMPRINC.type: empty text, an error.
- EFFECT.nominal_rate: empty text; EFFECT.npery: numeric text, empty text.
- MIRR.finance_rate: numeric text, empty text; MIRR.reinvest_rate: empty text, an error.
- NOMINAL.effect_rate: numeric text, empty text; NOMINAL.npery: numeric text, empty text.
- SYD.cost: empty text, a blank cell; SYD.salvage: numeric text, empty text, an error; SYD.life: numeric text, empty text, an error; SYD.per: numeric text, empty text, an error.
- AMORDEGRC.cost: empty text, an error; AMORDEGRC.date_purchased: empty text, an error; AMORDEGRC.first_period: empty text, an error; AMORDEGRC.salvage: empty text, an error, a blank cell; AMORDEGRC.period: empty text, an error; AMORDEGRC.rate: empty text, an error; AMORDEGRC.basis: empty text, an error.
- AMORLINC.cost: empty text, an error, a blank cell; AMORLINC.date_purchased: empty text, an error; AMORLINC.first_period: empty text, an error; AMORLINC.salvage: empty text, an error; AMORLINC.period: empty text, an error; AMORLINC.rate: empty text, an error; AMORLINC.basis: empty text, an error, a blank cell.
- DOLLARDE.fractional_dollar: empty text, an error; DOLLARDE.fraction: empty text, an error.
- DOLLARFR.decimal_dollar: empty text, an error; DOLLARFR.fraction: numeric text, empty text, an error.
- FVSCHEDULE.principal: empty text, an error.
- ISPMT.rate: empty text; ISPMT.per: empty text, an error; ISPMT.nper: numeric text, empty text, an error; ISPMT.pv: empty text, an error.
- PDURATION.rate: empty text; PDURATION.pv: empty text, an error; PDURATION.fv: empty text, an error, a blank cell.
- RRI.nper: numeric text, empty text, an error; RRI.pv: empty text, an error; RRI.fv: empty text, an error.
- VDB.cost: empty text, an error; VDB.salvage: empty text, a boolean, an error; VDB.life: numeric text, empty text, an error; VDB.start_period: numeric text, empty text, an error; VDB.end_period: numeric text, empty text, an error; VDB.factor: empty text, an error.
Write the probe case probes/m9-fin-annuity-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-fin-annuity-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/financial.md "Financial function argument modes", citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The securities functions (T-3125).
- `array`-mode parameters (NPV values, IRR, MIRR values, XIRR and XNPV values and dates, FVSCHEDULE schedule): T-2103 and T-2118 probe those.
- Numeric last digits of the probed values (T-2730, T-3014 and the other PMT/IPMT tasks).
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
