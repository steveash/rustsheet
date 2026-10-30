---
id: T-3020
title: "TAN and COT in an odd quadrant: fit Excel's reciprocal of a 64-bit tangent (lifts T-2147/T-2664 probes from 129/131 to 145/150)"
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
`excel_tan` (and COT through it) reproduces Excel's last bit where the pi64-reduced argument lies in an odd quadrant, so
the T-2147 and T-2664 near-tie probes keep at most their genuinely unexplained cells, or numbers.md says why the model was
not shipped.

## Context
Gap triage 2026-10-28 (eval run on origin/main 8d2f939). The 21 TAN misses of probes-numeric/t2147-tan-double-rounding and
the 19 COT misses of probes-numeric/t2664-cot-double-rounding (excel-graph; ungated suite, so no baseline lines) had no open
owner: T-2147 and T-2664 closed them as unpredictable near-tie noise (numbers.md "TAN misses this model by 1 ulp", the
T-2147 and T-2664 paragraphs). They are not noise. Two observations from the cached Excel values:

1. Every miss is at |x| >= 15.4784, where argument reduction happens; all 38 rows with |x| <= 1.6 match (COT's
   review already noted this for |x| <= 1.5).
2. 17 of the 21 TAN misses have an odd quadrant `n = round(x / (π₆₄/2))` (59 of the 150 rows are odd), where
   `tan x = -cot r`.

Model tried in this triage (exact rational arithmetic, Python `fractions` plus 80-digit `decimal` series for sin and cos
of the exact remainder `r = x - n·π₆₄/2`): for odd `n`, take the tangent of `r` rounded to a 64-bit significand first,
then its reciprocal, `t = -1 / RN64(tan r)`, and round the result as today, `RN53(RN64(t))`; even `n` unchanged. This
matches **145 of 150** TAN cells (shipped model: 129) and, with COT as today's twice-rounded `1/TAN`, **145 of 150** COT
cells (shipped: 131). The remaining misses in both probes are rows 73, 106, 133, 135 and 145; row 133 (n = -41, odd) is
the only cell the shipped model gets right that the new one loses. Reducing with a 66-bit, 68-bit or exact π instead of
π₆₄ drops to 48 of 150, so π₆₄ stays. Rounding `r` itself to 64 bits changes nothing (for these x it is exact in 64 bits).
Rounding `RN64(sin r)/RN64(cos r)` for even `n` gives 95, and `1/RN64(cot r)` for even `n` gives 118, so the even quadrant
keeps the plain tangent. This is an x87 FPTAN-like shape (a 64-bit tangent of the reduced argument, then the reciprocal for
the odd quadrant) and is consistent with "Excel's own tangent carries an error up to about half a 64-bit ulp" only in the
odd quadrant.

Examples (`make eval-show CASE=probes-numeric/t2147-tan-double-rounding.case.json`):

| case / cell | formula | x | n | Excel | shipped | odd-quadrant model |
|---|---|---|---|---|---|---|
| t2147 Calc!B8 | `=TAN(A8)` | -26.3054 | -17 | -2.3775582498320422 | -2.3775582498320427 | Excel's |
| t2147 Calc!B107 | `=TAN(A107)` | 8242 | 5247 | -31.5617681655554 | -31.561768165555403 | Excel's |
| t2147 Calc!B22 | `=TAN(A22)` | -5329990 | -3393177 | 31.011364765900876 | 31.011364765900872 | Excel's |
| t2147 Calc!B145 | `=TAN(A145)` | 15.4784 | 10 | -0.2336827343629212 | -0.23368273436292122 | still misses (even n) |
| t2664 Calc!B8 | `=COT(A8)` | -26.3054 | -17 | -0.42059957945116294 | -0.4205995794511629 | Excel's |

TAN misses (rows of Calc!B): 8, 22, 25, 49, 58, 59, 66, 73, 81, 106, 107, 111, 115, 123, 128, 130, 132, 135, 140, 145, 149.
COT misses: the same minus 25 and 123. Code: `excel_tan` in crates/rustsheet-engine/src/functions/math_and_trigonometry/trig.rs
(today `dd_rounded_twice(sin/cos)` of `excel_sin_cos_exact`), `reciprocal_trig` in mathematical.rs for COT. IMTAN and IMCOT
of a real number are said to take TAN's kernel (T-2774, blocked; engineering.md), so check their real-argument cells too.

Open questions for the fit (try on the evidence already cached; no new oracle run needed to ship or reject):
- Rows 73, 106, 135, 145 (even n) and 133 (odd n): a guard-bit variant of the 64-bit tangent (65..68 bits, rounded or
  truncated), an octant split at |r| > π/4·½ (FPTAN-style `tan(π/4 - y)` identities), or genuine Excel noise.
- Whether `RN64(tan r)` must be rounded or truncated, and whether the reciprocal is rounded to 64 bits before the final
  RN53 (in this triage, rounding it again to 64 bits gave the same 145).

## Acceptance criteria
- [ ] numbers.md "TAN misses this model by 1 ulp" records the odd-quadrant model with its counts on every cached TAN and
      COT cell with a constant argument (the 311 cells of T-2147's count, t2147, t2664, trig-odd-even-sumxmy2-sweep,
      m9-trig-reduction, probes/trig-huge-argument, gen-m5, gen-m6, gen-m9), against the shipped model's, listing
      every cell the new model loses.
- [ ] If it loses no cell outside row 133 of the two near-tie probes (or row 133 is explained): `excel_tan` uses it,
      probes-numeric/t2147-tan-double-rounding passes at least 145 of 150 cells and t2664-cot-double-rounding at least
      145 of 150, with unit tests on rows 8, 22 and 107 (TAN) and 8 (COT) quoting the Excel values above.
- [ ] No new known failure in any gated suite (`make eval-baseline` refuses regressions; m9-trig-reduction 666/666 and
      probes/trig-huge-argument 86/86 stay whole); SIN, COS, SEC and CSC cells unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMTAN's complex double-angle form (T-0907) and its blocked real-argument case (T-2774).
- The π₆₄ reduction and the 2^27 `#NUM!` cut (numbers.md "Trig of a huge angle", verified).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
