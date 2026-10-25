---
id: T-2666
title: "PPMT whose discount is exactly 2^-1023 past the range: why PPMT(7,10,350,58827) keeps it (gen-m9, 1 cell)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PPMT past `(1+rate)^nper`'s range has Excel's value where its discount `(1+rate)^(per-1-nper)` is subnormal, including
gen-m9 a91bad4e84f49739 Calc!A158.

## Context
T-2490 reads a discount below the normal range as 0 past the range (`discounted_principal`'s `past_range`,
crates/rustsheet-engine/src/functions/financial/annuity.rs; docs/excel-semantics/financial.md "IPMT cancels on fv").
probes/fin-ppmt-discount-underflow (excel-graph, 2026-10-25) backs that for period 1: `PPMT(r,1,300,1)` keeps a discount of
2^-1022 (B2) and is 0 from 2^-1022.5 on (B3:B14), and `PPMT(7,1,341,1)` (discount 2^-1023) is 0 (F1), as are 14 cells of
probes/fin-ipmt-ppmt-huge-rate (B4:B12, A817:A821). But `PPMT(7,10,350,58827)` = gen-m9 a91bad4e84f49739 A158 (Data!B8 is 7)
is -4.581304695604488E-303 = -58827*7*2^-1023 in Excel, its discount 8^-341 = 2^-1023 kept; ours 0. Period 1 versus a later
period, or the exact power of two of `1+rate = 8`, may separate them. Probe later periods with subnormal discounts at
power-of-two and other bases (e.g. `PPMT(7,k,340+k,1)`, `PPMT(3,k,510+k,1)`, `PPMT(10,k,295+k,1)` for k = 2, 10, 100) and
fit the rule.

## Acceptance criteria
- [ ] Visible parity (gen-m9): the known-failure line of gen-m9/a91bad4e84f49739.case.json Calc!A158 disappears from
      eval/baseline/gen-m9.txt, with no new known-failure lines in any baseline.
- [ ] The probe and its rule in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IPMT past the range (T-2665).
