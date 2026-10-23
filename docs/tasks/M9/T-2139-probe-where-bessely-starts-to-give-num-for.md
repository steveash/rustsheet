---
id: T-2139
title: "Probe where BESSELY starts to give #NUM! for large orders and Excel's BESSELJ order cap"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for two unprobed BESSEL order limits: the first order n where `BESSELY(x,n)` is `#NUM!`, and whether `BESSELJ` has any order cap. Both are recorded in `docs/excel-semantics/engineering.md`.

## Context
Found by T-0908's review. probes-numeric/m9-bessel (T-1004) shows `BESSELY(10,101)` finite and `#NUM!` at n = 1000, but nothing in between. It also shows `BESSELJ(x,1000000)` = 0, while RustSheet caps every order at 100,000 (`MAX_ORDER` in
`functions/engineering/transcendental/bessel_excel.rs`; "BESSEL order cap" in engineering.md is a DEVIATION).
Does Y's `#NUM!` come from overflow of the upward recurrence, giving +-inf, or from an explicit cap? Bisect n in 101..1000 at x = 1, 10 and 50.

## Acceptance criteria
- [x] A probes case (tag M9) that bisects the first `#NUM!` order of BESSELY at x = 1, 10, 50, oracle-filled.
- [x] A few BESSELJ cells at orders 100001..2^31-1 (e.g. `BESSELJ(10,2^31-1)`) oracle-filled. engineering.md states what Excel does there.
- [x] Any engine mismatch is filed as a follow-up task or fixed. The order cap stays unless it needs a decision record.
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising or removing `MAX_ORDER` (directive 5).

## Outcome
Probe `probes/m9-bessel-order-limits` (excel-graph filled). BESSELY's first `#NUM!` is overflow: n = 152 (x=1), 247 (x=10), 401 (x=50); the engine already matches. BESSELJ's real cap is 10,000,000 (`#NUM!` above, nonzero values below, e.g. `BESSELJ(1E5,100001)`); the engine's 100,000 cap stays (directive 5), seven cells recorded as decision-0014 deviations in `eval/suites/probes/deviations` (decision 0104). BESSELK orders probed too. Recorded in `docs/excel-semantics/engineering.md`.

## Handoff (resumed)
Probe, oracle sidecar, `engineering.md` findings, baseline and all four reviewer approvals are done on this branch (PR #1648); `make check` passes locally. CI `hygiene` fails `check_governance.py`: `eval/suites/probes/deviations` is a protected path and needs a decision record in `docs/decisions/` (`make new-decision`). The 7 lines I added there (BESSELJ orders 100,001..10,000,000 where the engine's directive-5 cap gives `#NUM!`) are standard decision-0014 deviations. Next: write the decision record (or have it cite 0014), push, re-run CI, re-review, set `status: done`, merge. Alternative if a record is unwanted: drop those 7 lines and let `make eval-baseline` record them as known failures instead.
