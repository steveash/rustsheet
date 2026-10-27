---
id: T-2920
title: "IMSEC's last two im-sec-csch-fused texts (B3, B31) and their IMREAL (C3, C31): Excel's 15 digits, no older IMSEC cell lost"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
IMSEC prints Excel's 15 digits on the last two probes-numeric `im-sec-csch-fused` texts T-2884 left off, B3 and B31,
so their IMREAL cells C3 and C31 pass too, with no older IMSEC oracle cell lost.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main f29502d9): these four cells are the only unowned visibly-off cells
of the functions-engineering lane (T-2884, now done, owned them; its Outcome left B3 and B31 one unit of the 15th digit
low in the real part). probes-numeric/im-sec-csch-fused (excel-graph, 2026-10-27):

| cell | formula | Excel | ours | exact (mpmath, 40 digits) |
|---|---|---|---|---|
| S!B3 | `IMSEC("2.11217+0.627617i")` | -0.86864314680255+0.803795556920423i | -0.868643146802549+0.803795556920423i | -0.86864314680254958074 + 0.80379555692042345171i |
| S!C3 | `IMREAL(B3)` | -0.86864314680255 | -0.868643146802549 | |
| S!B31 | `IMSEC("-3.75483+1.33847i")` | -0.436112042213995+0.267414799382832i | -0.436112042213994+0.267414799382832i | -0.43611204221399458975 + 0.26741479938283217917i |
| S!C31 | `IMREAL(B31)` | -0.436112042213995 | -0.436112042213994 | |

Both real parts sit near a 15-digit rounding tie (…549|58 and …994|59): Excel is correctly rounded here and ours
lands a few ulps low. That is not the `accuracy` stage: elsewhere IMSEC's Excel kernel is visibly inaccurate
(`IMSEC("956.615+0.00387493i")`, off in the 10th digit, engineering.md "IM* reciprocal kernels"), so the target is
Excel's algorithm, not a more accurate one. Current kernel: `imsec` and `sec_sinh_cosh` in
crates/rustsheet-engine/src/functions/engineering/complex/trig.rs, `t = cos²x + ((cosh 2y + 1)/2 − 1)`, numerators
`cos x·(cosh y/t)`, `sin x·(sinh y/t)`. engineering.md "IM* reciprocal kernels" has the scores of every candidate
T-2743, T-2757, T-2841 and T-2884 tried. Untried directions: the `cos x` / `cos²x` source (the x87-reduced
`trig_sin_cos` against `excel_sin_cos`, or cos²x as `(1 + cos 2x)/2`), the order of the two divisions, and the
`INVERSE_EXP_LOW`/`HIGH` band ends (y = 0.627617 and 1.33847 here; 2y is inside the band too).

Score every candidate on every IMSEC oracle cell (gen-m9, ironcalc-calc, probes, probes-numeric `im-*`) with a scratch
build outside the commit, as T-2884 did. Time-box the fit (decision 0056). If no candidate fits both cells without
losing an older one, record the scores in engineering.md and say whether a research-lane oracle sweep of near-tie
IMSEC inputs would separate the candidates (file it with `make new-task` if so).

## Acceptance criteria
- [x] probes-numeric/im-sec-csch-fused S!B3, S!B31, S!C3 and S!C31 pass, every other IMSEC oracle cell that passes
      today still passes, and no new F line appears in any baseline (`make eval-baseline`); or the task records,
      with the scores, that no candidate does, and files the follow-up above.
- [x] engineering.md "IM* reciprocal kernels" records the result and the scores.
- [x] `make check` green; reviewers (including Excel fidelity) approve.

## Out of scope
- IMCSCH, IMCSC, IMCOT and IMSECH kernels (all their fused-probe texts already fit).
- IMSEC of a real number (`real_trig`, T-2534).

## Visible-parity stage
S!B3, S!B31, S!C3 and S!C31 show Excel's 15 digits (stage `visible`: reproduce Excel's IMSEC formulation).

## Outcome
IMSEC's denominator takes sinh²y as `(cosh 2y − 1)/2` inside its parts' `1/u` band (0.01 < |y| <= 20), and keeps
`(cosh 2y + 1)/2 − 1` outside it (`imsec` in functions/engineering/complex/trig.rs). probes-numeric
im-sec-csch-fused S!B3, S!B31, S!C3 and S!C31 now pass (230 → 234 of 234). No IMSEC oracle cell in gen-m9,
ironcalc-calc, probes or probes-numeric is lost: rows 29 and 33 of im-last-digit-sweep, below the band, still
need the old form. The scratch-build scores of the losing candidates are in engineering.md "IM* reciprocal kernels"
(T-2920 bullet). These include x87-extended evaluation, fused and half-angle `t`, and extended numerators.
Eval delta: probes-numeric +4 cells (ungated suite); gated baselines unchanged. Follow-ups: none. Where the switch
lies is the band's own unprobed end, which T-2797 already covers.
