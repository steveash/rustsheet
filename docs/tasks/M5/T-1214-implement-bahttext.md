---
id: T-1214
title: "Implement BAHTTEXT"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BAHTTEXT returns Excel's Thai-language baht text for a number.

## Context
From T-1087's probe probes/text-modes-bahttext: `BAHTTEXT(5)` and `BAHTTEXT("5")` are "ห้าบาทถ้วน" in Excel and `#NAME?` in the engine. Probe more values (satang, millions, negatives, zero, large numbers) with the excel-probe skill before implementing.

## Acceptance criteria
- [ ] probes/text-modes-bahttext passes, and a wider probe covers satang, negatives and millions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`BAHTTEXT` implemented (functions/text/bahttext.rs) from wide oracle probes (text-bahttext-wide, -wide2, -form*, -text*): 15-significant-digit cut, decimal half-up satang, ลบ sign, เอ็ด/ยี่ rules, six-digit ล้าน grouping, number coercion and array lifting. Findings in docs/excel-semantics/text.md#bahttext. Probes: 5 known failures fixed (text-modes-bahttext C1/C2, availability-03 A3, and the new spill/range forms), +~110 new passing cells, 0 regressions. Unprobed: argument count 0 or 2.
