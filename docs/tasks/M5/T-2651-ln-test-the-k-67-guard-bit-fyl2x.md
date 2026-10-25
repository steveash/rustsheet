---
id: T-2651
title: "LN: test the k=67 guard-bit FYL2X model on the full T-1941 near-tie set and ship it if it never loses"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
`excel_ln` uses the 67-bit rounded FYL2X product (k=67 rn) if it matches Excel at least as well as the shipped model everywhere it has evidence, or a numbers.md note says why not.

## Context
Found by T-2144 (decision 0112; docs/research/t1941/ln_guard_bits.py): rounding the exact product `RN64(ln 2) * log2 x` to 67
bits before RN64 and RN53 matches 354 of T-2143's 357 fill-stable points (shipped model 345, and it loses none the shipped
model matches), all 4,000 sampled fixture points, 18 of the 30 fixture near-ties (shipped 0), and 242 of probes/ln-fyl2x-kernel's
253 cells (shipped 226: it fixes 17 of its 27 known failures and loses Calc!B231, x = 9.612345678912595, one of the 3
near-ties where that probe's fill and the 2026-10-19 research run already disagree, numbers.md). It was not shipped because it misses 3 fill-stable points
and the 4,000-point sample holds few near-ties, so it says little about a guard-bit model's risk. The deciding evidence is the
near-tie subset of the full T-1941 rows (2.89M cells; `RUSTSHEET_T1941_ROWS`, docs/research/t1941/extract_runs.py on branch
agent/research/T-1941-crack-gammaln-gamma-ln): every x whose product is within 4e-4 ulp53 of a 53-bit midpoint. Also check
the T-2133 ASINH/ACOSH cells (probes-numeric/t2133-asinh-acosh-ln) and the LOG10 and GAMMALN probes that call the shared
FYL2X path (`x87_log10.rs`).

## Acceptance criteria
- [ ] numbers.md "LN is FLDLN2 then FYL2X" records k=67 rn's count against the shipped model's on the T-1941 near-tie subset,
      with its losses (points the shipped model matches and k=67 misses).
- [ ] If k=67 rn loses no fill-stable point anywhere (a losing point is excluded only when a T-2143-style refill of it, three fills, shows Excel's bit varies, as B231's two runs already do): `excel_ln`
      uses it (LN only: LOG10's FYL2X constant differs and was not fitted; the LOG10, GAMMALN and ASINH/ACOSH probes must not
      regress), the fixture is
      regenerated, and the probes/ln-fyl2x-kernel lines it fixes leave eval/baseline/probes.txt. B231's new known failure needs a decision record
      (decision 0112 amended) citing its fill instability, since a gated regression is otherwise not allowed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA and GAMMALN model shipping (T-2128, T-2129).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
