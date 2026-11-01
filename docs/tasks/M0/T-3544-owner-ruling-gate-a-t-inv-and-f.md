---
id: T-3544
title: "Owner ruling: Gate A T.INV and F.INV points where Excel is more than G off outside the R3 region"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-3358]
---

## Goal
Get an owner ruling (decision 0188 R3) on the T.INV and F.INV Gate A points where Excel is more than G = 1e-4 off the
math but the point is outside the approved tiny-p region, or cannot be marked. The ruling either widens the region with
a decision record and R3 marks, or keeps Excel as the target. The record says which.

## Context
- T-3358's tables have 17 points where Excel is more than G off the math.
  - Decision 0213 marks 8 of them, the ones inside the approved region (T.INV and F.INV, p < 1e-150).
  - T-3386 owns the two-tailed points: T.INV.2T A98, A99 and A103, and TINV A102.
  - This task owns the rest:
    - accuracy/T.INV/s1 A91 (p 8.2E-131, df 66741) and A96 (p 2.9E-116, df 91643);
    - accuracy/F.INV/s1 A164 (p 1.9E-93) and A169 (p 1.2E-69);
    - accuracy/F.INV/s1 A167, `F.INV(4.36982683607437E-189,1,1698856)`, discussed below.
- A167 (p 4.4E-189 at (1, 1698856)) is inside decision 0188's p < 1e-150 bound but outside the region the entry
  names, and it cannot simply be marked:
  - The true quantile is 3.0E-377, below the double range.
  - Excel returns 3.85E-145 and the engine returns 0.
  - The point passes Gate A only through the absolute floor of 1e-300.
  - An R3 mark would make 0 the target, and that choice is the owner's.
- None of these points blocks anything:
  - On all of them the engine is within ε of the math (for A167 it returns 0), so they pass Gate A through its math
    side today, and nothing waits on this ruling.
  - Until the ruling, directive 1 keeps Excel as the target.
- Background: statistics.md, "Tiny-p quantiles: Excel is grossly wrong, RustSheet returns the math (MATH-OVER-EXCEL)".

## Acceptance criteria
- [ ] The owner's ruling on each point is recorded: in a decision record, or in this task's Outcome when Excel stays
      the target.
- [ ] If the ruling widens the region:
  - the points are marked in `eval/accuracy/math-over-excel.tsv`;
  - the statistics.md entry's region is updated.
- [ ] If Excel stays the target, the engine-side work is filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The two-tailed points (T-3386).
- Changing the engine.
