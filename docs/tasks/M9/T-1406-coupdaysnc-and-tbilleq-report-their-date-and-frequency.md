---
id: T-1406
title: "COUPDAYSNC and TBILLEQ report their date and frequency #NUM! before a later argument's error or text"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
COUPDAYSNC (and the coupon and T-bill functions that share its argument checks, per the probe) report an invalid
frequency or date order as `#NUM!` before a later argument's error value or non-numeric text, as Excel does.

## Context
Kernel map (T-1522, decision 0056): rank 15 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/0c7fc447bc2eebfd Calc!A24 `=COUPDAYSNC(37342,46362,ROUND(Data!$E$4,10),(FLOOR(Data!C3,Data!E$9)+(Data!E11/Data!A9)))`:
  frequency -0.0001 (invalid), basis `#DIV/0!` (E11/A9 = 0.1/0). Excel `#NUM!`, engine `#DIV/0!`.
- gen-m9/4df1b22d3ed63f58 Calc!A20 `=(TBILLEQ(38887,(1e-15-Data!B12),TEXTSPLIT("a*",Data!A1:D2))=Data!$B10)`: maturity is
  1e-15-2 (Data!B12 = 2), a negative and invalid serial; discount is `TEXTSPLIT("a*",A1:D2)` (text, or an error with D2
  blank). Excel `#NUM!`, engine `#VALUE!`.
So these Analysis ToolPak functions validate frequency and the settlement/maturity order before they read later
arguments. Opus: the order is evidenced only for these two shapes. Probe with the excel-probe skill the general order
for COUPDAYSNC (and one more of COUPDAYBS/COUPNCD/COUPNUM/COUPPCD), TBILLEQ, TBILLPRICE and TBILLYIELD: an error in each
argument position combined with an invalid frequency, a reversed date pair and text, and record the rule in
docs/excel-semantics/financial.md. Code: `fn_coupdaysnc` (functions/financial/financial_bonds.rs) and `fn_tbilleq`
(functions/financial/mod.rs); find the shared argument reader before changing several functions.
Semantics: docs/excel-semantics/financial.md "Analysis ToolPak securities functions check frequency and dates first".

## Acceptance criteria
- [ ] gen-m9/0c7fc447bc2eebfd A24 and gen-m9/4df1b22d3ed63f58 A20 pass in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The error-order probe cells pass; financial.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
