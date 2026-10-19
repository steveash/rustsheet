---
id: T-1942
title: "Probe Z.TEST where sigma/sqrt(n) underflows or the data sum overflows"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Z.TEST's two unprobed numeric edges return what Excel returns, backed by oracle cells.

## Context
`functions/statistical/z_test.rs` (T-1929) keeps two guards with no Excel evidence:
- a sigma/sqrt(n) that underflows to 0 in plain doubles is `#DIV/0!` (kept from the IronCalc fork), e.g.
  `=Z.TEST({1,2},1,5E-324)` and `=Z.TEST({1,2},1,1E-320)`;
- a non-finite p (an omitted sigma with a data sum that overflows, e.g. `=Z.TEST(A1:A3,0)` over 9E307, 9E307, 1) is
  `#NUM!`. The guard also turns an omitted sigma that overflows to infinity into `#NUM!`:
  `=Z.TEST(C1:C2,-9E307)` over 9E307, 8E307 was 0.5 before T-1929 (inf/inf z) and is `#NUM!` now.
Probe these (excel-probe skill, one probes case, one milestone tag), plus a sigma whose x87 z overflows
(`=Z.TEST({1,2},0,1E-308)`), and held-out cells for the z formula (random data, n not a perfect square, sigma
given and omitted, x not 0) that separate z = fl(fl(mean-x)*sqrt64(n))/sigma from the textbook
(mean-x)/(sigma/sqrt(n)) and the plain-double (mean-x)*sqrt(n)/sigma at the p level. Add GAMMA.DIST with a
subnormal x/b (`=GAMMA.DIST(2E-300,0.5,1E8,TRUE)`, `=GAMMA.DIST(1E-300,0.5,1E10,TRUE)`): T-1929 keeps the
subnormal quotient where a worksheet `/` would flush it. Match the engine to the oracle; record the rules in
docs/excel-semantics/statistics.md (the T-1778 section, "Engine (T-1929)").

## Acceptance criteria
- [ ] The probe case is filled from excel-graph and passes.
- [ ] The two guards in `z_test.rs` cite the probe cells or are changed to match them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Z.TEST argument coercion (T-1813).
