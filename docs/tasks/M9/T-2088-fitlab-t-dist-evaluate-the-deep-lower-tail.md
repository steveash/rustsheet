---
id: T-2088
title: "fitlab t-dist: evaluate the deep lower tail without rounding y = n/(n+t^2)"
milestone: M9
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`uv run rustsheet-fitlab t-dist` scores the "correctly rounded" candidate against the true Student-t
CDF at every df, so a fit no longer reads a fit-lab artefact as an Excel quirk.

## Context
Found by T-0969's second attempt (2026-10-20). `t_cdf` in
`tools/src/rustsheet_tools/fitlab/kernels/beta.py` takes `fn(y, n/2, 0.5)/2` for a lower tail under
0.1 on its y <= z branch, with `y = n/(n + t^2)` rounded to a double first. At n = 1E6, `1 - y` is about 4E-6, so that
rounding moves the result far more than any I_x algorithm does: the "correctly rounded" candidate
misses `T.DIST(-2,1E6,TRUE)` by 152,191 ulp and `T.DIST(-10,1E6,TRUE)` by 43,561 ulp. Both figures
reproduce exactly by evaluating the exact I_x (mpmath, 60 digits) at the rounded y. At the exact
y, Excel is within 1 ulp on the first cell and 38 ulp on the second. The `Ibeta` signature passes
only `x`, so the mapping cannot hand the kernel `1 - y` exactly. Either give the candidates
`(x, 1 - x)` as Cephes' `incbet` callers do, or map the deep lower tail through
`0.5 - I_z(1/2, n/2)/2` evaluated exactly inside the reference.

## Acceptance criteria
- [ ] The "correctly rounded" candidate is 1 ulp from Excel on `T.DIST(-2,1E6,TRUE)` and 38 ulp on
      `T.DIST(-10,1E6,TRUE)` (probes-numeric m9-t-dist-cdf A149, A150; the exact CDF's distances),
      shown in the fit-lab output.
- [ ] The other candidates' scores are re-run and the new table is pasted in the PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine's T.DIST kernel (T-0969).
