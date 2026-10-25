---
id: T-2144
title: "LN at FYL2X near-ties: fit Excel's last bit or settle the 27 probes/ln-fyl2x-kernel known failures"
milestone: M5
lane: functions-math
status: done
priority: P3
size: M
tier: opus
depends_on: [T-2143]
---

## Goal
The 27 known failures of probes/ln-fyl2x-kernel (LN is catalog M5) get an owner and a decided outcome. Either the
engine gives Excel's last bit at FYL2X near-ties, or a decision record shows that Excel's bit there is not a function
of x and says how the probe scores those cells.

## Context
- Needs an oracle sweep (T-2143, done: probes-numeric/t2143-ln-near-ties-fill1..3; the fills disagree at 23 of 380 points, so Excel's bit at a near-tie is not a fixed function of x, numbers.md). The cached evidence is the T-1941 runs (30 misses in 2.9M cells) and the probe's
  single fill, which disagree at 3 of the 30 points. That is too few points to fit a near-tie rule blind.
- Failing cells: probes/ln-fyl2x-kernel Calc!B10, B107, B113 and 24 others (eval/baseline/probes.txt). Each is one
  ulp, for example `LN(A10)`: Excel -1.0685588514970168, engine -1.068558851497017. `make eval-show
  CASE=probes/ln-fyl2x-kernel.case.json` lists them all.
- Model today (numbers.md "LN is FLDLN2 then FYL2X", T-2123): `RN53(RN64(RN64(ln 2) * log2 x))`, with the product
  in double-double. In `crates/rustsheet-engine/src/x87_log10.rs` (`excel_ln`) it shares the FYL2X evaluation with
  LOG10. `test/fixtures/ln_kernel.fixture` pins it on 4,000 points plus the 30 misses.
- Hypotheses to test against T-2143's fills:
  1. Real FYL2X is not correctly rounded to 64 bits. Its internal precision, about 67 bits on Intel, decides
     near-ties of the 64-bit result. Model the product with a few guard bits and a truncation, not an exact RN64.
  2. The servers behind the excel-graph oracle differ (Intel against AMD FYL2X). Then fills disagree at the same x,
     and no single model fits.
- If hypothesis 1 fits every fill, LOG10 (`excel_log10`, same FYL2X path) and GAMMALN/GAMMA (T-2128, T-2129, which
  call `excel_ln`) inherit it. Check that their probes do not regress.

## Acceptance criteria
- [x] numbers.md "LN is FLDLN2 then FYL2X" records the fit: each candidate's count on T-2143's case, on the 30
      near-ties and on the committed ln_kernel fixture.
- [ ] (n/a, branch not taken: no candidate fits) If a candidate matches every point on which all of T-2143's fills agree, and keeps every fixture point:
      `excel_ln` uses it. The probes/ln-fyl2x-kernel known-failure lines it fixes then disappear from
      eval/baseline/probes.txt, with no new ones in any gated suite.
- [x] Otherwise: a decision record (`make new-decision`) says how the probe scores a near-tie whose Excel bit
      varies between fills. One option is to accept either neighbour, as decision 0042 accepts its pair. The other is
      to keep them as named known failures. The record also says whether these cells count toward any milestone exit.
      No baseline hand-edit and no `--allow-regressions`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The LN cost (perf, T-2016 covers LOG10).
- GAMMALN/GAMMA model shipping (T-2128, T-2129).

## Outcome
Fitted guard-bit FYL2X models (a k-bit internal product, truncated or rounded, before RN64 and RN53; k = 66..72;
docs/research/t1941/ln_guard_bits.py). They were scored against the 357 points where T-2143's three fills agree, the
`ln_kernel` fixture and probes/ln-fyl2x-kernel; numbers.md has the table. The best, k = 67 rounded, matches 354 of 357 (the shipped
model 345), all 4,000 sampled fixture points, 18 of the 30 fixture near-ties and 242 of the probe's 253 cells (shipped 226).
No candidate fits every stable point, so `excel_ln` is unchanged. Decision 0112 keeps the 27 probes/ln-fyl2x-kernel
near-ties as named known failures with no exit exemption. Follow-up T-2651 tests k = 67 rounded on the full T-1941 near-tie set and
ships it if it loses no point. Eval: no change.
