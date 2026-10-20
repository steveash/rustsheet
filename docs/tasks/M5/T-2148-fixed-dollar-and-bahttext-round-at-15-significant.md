---
id: T-2148
title: "FIXED, DOLLAR and BAHTTEXT round at 15 significant digits first (ROUND's stage 1)"
milestone: M5
lane: functions-text
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
FIXED, DOLLAR and BAHTTEXT round as ROUND does: cut to 15 significant digits first (a 16th-digit tie toward zero), then round half away at `decimals`, printing zeros past the 15th digit; DOLLAR with negative `decimals` prints no parentheses for a value that rounds to 0.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_fixed`/`fn_dollar` (crates/rustsheet-engine/src/functions/text/string_format.rs) round the binary value, so `DOLLAR(1.005)` is
"$1.00" and 2^53 prints all 16 digits. BAHTTEXT already does the 15-digit cut but breaks a 16th-digit tie away from zero.
Reuse ROUND's stage 1 (`crates/rustsheet-engine/src/decimal_round.rs`); do not add a new rounding helper. Rule:
docs/excel-semantics/text.md "FIXED, DOLLAR and BAHTTEXT cut to 15 significant digits first" (added by this triage).
The three `DOLLAR(x,-1)` cells are the sign rule (value rounds to 0 at negative decimals: "$0", not "($0)").

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 15 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/0362d48f56108faa Calc!A43 | `=FIXED(INT(2^53+1),0)` | "9,007,199,254,740,990" | "9,007,199,254,740,992" |
| gen-m5/0362d48f56108faa Calc!A119 | `=DOLLAR(SUBTOTAL(9,Data!A5:D7,Data!C1:D10))` | "$3,000,000,123,456,800.00" | "$3,000,000,123,456,798.50" |
| gen-m5/0362d48f56108faa Calc!A381 | `=DOLLAR(Data!E9)` | "$1.01" | "$1.00" |
| gen-m5/2abdc1524e5aa685 Calc!A221 | `=FIXED((Data!A$12-TYPE(Data!B10)))` | "999,999,999,999,998.00" | "999,999,999,999,998.13" |
| gen-m5/2abdc1524e5aa685 Calc!A234 | `=DOLLAR(SUM(Data!E5:E10,SUMIF(Data!A2:D6,"a*"),Data!A6:A12))` | "$1,000,000,000,000,040.00" | "$1,000,000,000,000,041.50" |
| gen-m5/3b813881f835cd90 Calc!A48 | `=DOLLAR(1.005)` | "$1.01" | "$1.00" |
| gen-m5/3b813881f835cd90 Calc!A310 | `=DOLLAR(Data!C8)` | "$1.01" | "$1.00" |
| gen-m5/461d1c8dbb044769 Calc!A249 | `=DOLLAR((Data!B6-SUM(42,Data!C8:C8,Data!C9:C9)))` | "$999,999,999,999,955.00" | "$999,999,999,999,954.88" |
| gen-m5/50d8a085335c05e6 Calc!A158 | `=DOLLAR((ROUNDDOWN(9999999999999999,8)/2))` | "$4,999,999,999,999,990.00" | "$4,999,999,999,999,995.00" |
| gen-m5/ae9ed30812c4d62c Calc!A302 | `=FIXED((INT(2^53+1)--Data!E8),4)` | "9,007,199,254,740,990.0000" | "9,007,199,254,740,994.0000" |
| gen-m5/eb942f4a59b1c004 Calc!A90 | `=FIXED(SUMX2MY2((Data!B6-1),Data!B9:B9))` | "15,241,578,533,637,300.00" | "15,241,578,533,637,312.00" |
| gen-m5/399071ff6782ab16 Calc!A316 | `=DOLLAR((COUNTA(Data!B2,Data!B7:B7,1)/(Data!$B6+Data!A10)),-1)` | "$0" | "($0)" |
| gen-m5/5704666097cbc5b2 Calc!A274 | `=DOLLAR(((1.005/(-0.5))*Data!$D$6),-1)` | "$0" | "($0)" |
| gen-m5/ae9ed30812c4d62c Calc!A183 | `=DOLLAR(SUM((Data!C$11+Data!D5)),-1)` | "$0" | "($0)" |
| gen-m5/4d099bea7898ebf1 Calc!A66 | `=BAHTTEXT(((1000000000000000.0+Data!B1)-(1e-15*Data!E8)))` | "เก้าร้อยเก้าสิบเก้าล้านเก้าแสนเก้าหมื่นเก้าพ..." | "เก้าร้อยเก้าสิบเก้าล้านเก้าแสนเก้าหมื่นเก้าพ..." |

## Acceptance criteria
- [ ] These 15 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] text.md entry marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Values past 2^64 and very many decimals (T-2149); array lifting (T-2160).
