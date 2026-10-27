---
id: T-2917
title: "Probe whether AMORDEGRC's first-period subtraction from the cost rounds twice"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2849]
---

## Goal
The subtraction of AMORDEGRC's rounded first-period charge from the cost (`remain` before the walk) follows Excel,
backed by probe cells that tell a plain double difference from a twice-rounded one.

## Context
T-2849 found that Excel's AMORDEGRC walk rounds each `remain - charge` twice (x87 extended, then double;
docs/excel-semantics/financial.md "AMORDEGRC rounds its charge like ROUND") and applied the same `extended_sum` to the
first subtraction `cost - first_depr` (functions/financial/amortization.rs `Asset::degressive`) by analogy: no probed
cell tells the two apart. A distinguishing cell needs a cost whose `cost - first_depr` lands within half an extended ulp
of a double midpoint (so the two roundings give different doubles) and whose period-1 charge `rate * remain` then sits
within an ulp of a 15-significant-digit cut boundary, so the one-ulp difference shows in the result (as
probes/amordegrc-walk-drift D1-D12 did for the final rounding). Search for such costs with a scratch script (e.g. C with
`long double` on x86-64 Linux, which is x87 extended), then probe them with the excel-probe skill.

## Acceptance criteria
- [ ] A probe case holds at least 6 such cells (both directions), filled by the oracle, and every cell passes.
- [ ] financial.md records which subtraction Excel uses and drops "by analogy" from the entry and the code comment.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The walk after period 1 (T-2849) and past 2^20 periods (T-2916).
