---
id: T-2884
title: "Fit IMSEC's last seven im-sec-csch-fused texts (cosh y and sinh y parts)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMSEC prints Excel's 15 digits on the seven probes-numeric `im-sec-csch-fused` texts T-2841 left off, with no older
oracle cell lost.

## Context
T-2841 moved IMSEC to `cos x·(cosh y/t)`, 27 of 34 fused texts, and left B3, B9, B19, B20, B21, B31 and B33 one unit
of the 15th digit off (B33 is `IMSEC("-1.44771+0.0419247i")`, Excel 7.30033916775268-2.47256566938037i).
engineering.md "IM* reciprocal kernels" has the scores. The cosh y and sinh y parts fit more fused texts but lose
older cells: IMCOS's `trig_sinh_cosh` 31 of 34 but 9 older cells lost, the EXP pair 29 and 10 lost, IMSINH's `1/u`
form 28 and 1 lost. So Excel's IMSEC probably builds cosh y and sinh y in a way none of these match, maybe by band
(`INVERSE_EXP_LOW`/`HIGH`-like switches) or per sign of y. Start from `trig_sinh_cosh` and look at the older cells it
loses (probes-numeric `im-last-digit-sweep`). Score every candidate on every IMSEC oracle cell (gen-m9,
ironcalc-calc, probes, probes-numeric `im-*`), as T-2841 did with a scratch build outside the commit.
Also probe IMCSCH with a large real part and y = 0 (`IMCSCH("400")`, `IMCSCH("710")`), carried over from T-2841's
Context: past `inverse_exp_sinh_cosh`'s band its parts are libm's, and only `IMCSCH("710.4+0i")`
(probes/eng-im-overflow-edges A64) covers the overflowed `t`.
Code: `imsec` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] IMSEC fits more than 27 of the 34 fused texts and loses no older oracle cell, or the task records that no
      candidate does, with the scores.
- [ ] engineering.md "IM* reciprocal kernels" records the scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMCSCH kernel changes beyond that probe (all 44 fit since T-2841), IMCSC, IMCOT, IMSECH.
