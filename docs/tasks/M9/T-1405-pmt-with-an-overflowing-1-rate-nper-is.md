---
id: T-1405
title: "PMT with an overflowing (1+rate)^nper is a number, not #NUM! (gen-m9 3b8e9078261b5bd2 A34)"
milestone: M9
lane: functions-financial
status: in-progress
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
- [ ] gen-m9/3b8e9078261b5bd2 A34 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The overflow probe cells pass; financial.md records the rule (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
