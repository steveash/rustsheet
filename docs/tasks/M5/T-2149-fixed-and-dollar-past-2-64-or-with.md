---
id: T-2149
title: "FIXED and DOLLAR past 2^64 or with more than 20 decimals print 15 digits then zeros"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FIXED and DOLLAR print values past 2^64, and more than 20 decimals, as 15 significant digits followed by zeros, never a saturated integer.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Ours prints 18,446,744,073,709,551,615 (u64::MAX) for the integer part of DOLLAR(1E+30) and appends it as the fraction of FIXED
with 24 or 100 decimals: an integer cast saturates in `fn_fixed`/`fn_dollar` (string_format.rs) or their shared digit formatter.
Rule: docs/excel-semantics/text.md "FIXED, DOLLAR and BAHTTEXT cut to 15 significant digits first" (last paragraph). Build the
digits from the 15-digit decimal (T-2148's helper) rather than from integer casts. Directive 5: up to 127 decimals and 309 integer
digits must not allocate or loop unboundedly.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 5 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/399071ff6782ab16 Calc!A122 | `=FIXED(SUBTOTAL(1,Data!A8:A8,Data!D9:D10,Data!A10:D12),IFERROR((Data!E9*(-0.0001)),AVERAGE(Data!E5...` | "90909090909108.900000000000000000000000" | "90909090909108.000018446744073709551615" |
| gen-m5/6b831f5e2a20bc34 Calc!A232 | `=DOLLAR((SUMPRODUCT(Data!C5:E9)*Data!D7),-2)` | "$1,000,000,123,456,790,000,000,000,000,000" | "$18,446,744,073,709,551,615" |
| gen-m5/b0ecb7ca78460b77 Calc!A128 | `=FIXED(ASINH((1+123456789.123)),100,(COUNT(Data!D7:D8,(-0.5))<>YEAR(21835)))` | "19.32454895582430000000000000000000000000000..." | "19.00000000000000000000000000000000000000000..." |
| gen-m5/e0087a1fb9a36188 Calc!A291 | `=DOLLAR(SUBTOTAL(11,Data!B2:E12,Data!A1:D1),3)` | "$43,388,429,752,065,200,000,000,000,000.000" | "$18,446,744,073,709,551,615.000" |
| gen-m5/fc81c2f704bef138 Calc!A366 | `=DOLLAR(SUBTOTAL(11,Data!B5:E12),0)` | "$122,448,977,072,310,000,000,000,000,000" | "$18,446,744,073,709,551,615" |

## Acceptance criteria
- [ ] These 5 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 15-digit cut of ordinary values (T-2148).
