---
id: T-2607
title: "Lifted quantiles: an element that refuses its arguments before the search is charged 1, so BETAINV over a 4x51638 array is #SPILL!"
milestone: M9
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A lifted BETA.INV (and the other `QUANTILE_WEIGHT` quantiles where it holds) charges an element 1 when the kernel refuses it
before the search (an error argument, a probability outside (0, 1), alpha or beta <= 0), so a large lift of refused elements
is not `#ERROR!` from the cost cap.

## Context
gen-m9/4250f880ab640c49 Calc!A237, `=BETAINV(1+2^-52,(-1),..,MAKEARRAY(4,DATE(1952,..),..),NORM.S.INV(1.5))`, is a 4 x 51,638
array in Excel, `#SPILL!` in a cell. Since T-2600 the MAKEARRAY is built and the lifting broadcast takes it, but every
element is charged `QUANTILE_WEIGHT` (2^18, lift/kinds/statistical.rs), which caps a lift at 512 elements, so ours is
`#ERROR!`. Here every element is refused before `beta_quantile` runs (`Model::beta_inv_of`: probability 1+2^-52 and alpha
-1 are `#NUM!`, the last argument is an error). A `KernelWork::ByValue` charge (lift/kinds/mod.rs `ValueWork`, as DB/VDB's
`Loop` and the coupon walks do) can see that from the element's values. arrays.md "Arrays past the grid".

## Acceptance criteria
- [ ] gen-m9/4250f880ab640c49 Calc!A237 passes and leaves eval/baseline.
- [ ] A lift of refused elements past 512 runs; one of valid elements past the cap is still `#ERROR!` (unit tests).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising the cost cap.
