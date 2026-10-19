---
id: T-1405
title: "PMT with an overflowing (1+rate)^nper is a number, not #NUM! (gen-m9 3b8e9078261b5bd2 A34)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PMT (and IPMT/PPMT if the probe shows the same) returns Excel's number when `(1+rate)^nper` overflows a double, instead of `#NUM!`.

## Context
Kernel map (T-1522, decision 0056): rank 1 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3b8e9078261b5bd2 Calc!A34 `=(CEILING.MATH(IF(FALSE,"1",Data!$A12))+FLOOR(PMT(100,184,439103),(Data!D$7/Data!$B6)))`:
  Excel `#DIV/0!`, engine `#NUM!`. The significance `Data!D$7/Data!$B6` is `#DIV/0!` (B6 blank) and CEILING.MATH(FALSE) is 0,
  so Excel's `#DIV/0!` means `PMT(100,184,439103)` is not an error there (FLOOR returns its first argument's error first);
  the engine's PMT is `#NUM!` because 101^184 overflows.
The value is not evidenced (the limit of `-pv*r*P/(P-1)` as P grows is -pv*r = -43910300). Opus: first probe with the
excel-probe skill `PMT(100,184,439103)`, `PMT(100,200,1)`, `PMT(1,2000,1)`, `IPMT`/`PPMT` of the same, `FLOOR(NA(),1/0)`
(to confirm the error order this inference rests on) and a case where `P` is finite but `P-1` rounds to `P`. The annuity
formulation is T-0901's (docs/excel-semantics/financial.md "Annuity evaluation order"); keep its fitted PV/FV shape.
Code: `fn_pmt` in crates/rustsheet-engine/src/functions/financial/mod.rs.

## Acceptance criteria
- [x] gen-m9/3b8e9078261b5bd2 A34 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] The overflow probe cells pass; financial.md records the rule (VERIFIED).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe probes/pmt-overflow (excel-graph, 2026-10-19), 40 scored cells, all pass: when `pv*P` overflows, PMT is the
  annuity quotient divided through by P (limit `-pv*rate`, `/(1+rate)` for type 1; fv vanishes), also for a finite
  P (101^153); IPMT/PPMT follow; PV/FV/NPER stay `#NUM!`; a P whose reciprocal overflows (0, 2^-1070) is `#NUM!` for
  PMT/IPMT/PPMT/CUMIPMT whatever fv and type; `FLOOR(NA(),1/0)` is `#N/A` (the A34 inference holds).
- `compute_payment` keeps the fitted quotient whenever it is finite (no finite result changes digits) and falls back
  to the divided-through form; financial.md entry rewritten (VERIFIED).
- gen-m9/3b8e9078261b5bd2 A34 passes (39/40 in that case; A13 was already failing; gen-m9 is ungated, no baseline file).
- B8 `PPMT(1,1,60,1)` = -2^-60 (ours 0) is recorded but unscored: a last-digit question, filed as a new hypothesis on
  T-1528.
- Eval delta: fixed 1 (gen-m9 A34, ungated), new coverage 40 passing cells (probes/pmt-overflow), regressions 0.
