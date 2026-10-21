---
id: T-2144
title: "LN at FYL2X near-ties: fit Excel's last bit or settle the 27 probes/ln-fyl2x-kernel known failures"
milestone: M5
lane: functions-math
status: todo
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
- Needs an oracle sweep (T-2143). The cached evidence is the T-1941 runs (30 misses in 2.9M cells) and the probe's
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
- [ ] numbers.md "LN is FLDLN2 then FYL2X" records the fit: each candidate's count on T-2143's case, on the 30
      near-ties and on the committed ln_kernel fixture.
- [ ] If a candidate matches every point on which all of T-2143's fills agree, and keeps every fixture point:
      `excel_ln` uses it. The probes/ln-fyl2x-kernel known-failure lines it fixes then disappear from
      eval/baseline/probes.txt, with no new ones in any gated suite.
- [ ] Otherwise: a decision record (`make new-decision`) says how the probe scores a near-tie whose Excel bit
      varies between fills. One option is to accept either neighbour, as decision 0042 accepts its pair. The other is
      to keep them as named known failures. The record also says whether these cells count toward any milestone exit.
      No baseline hand-edit and no `--allow-regressions`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The LN cost (perf, T-2016 covers LOG10).
- GAMMALN/GAMMA model shipping (T-2128, T-2129).
