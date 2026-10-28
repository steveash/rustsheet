---
id: T-2714
title: "MOD of two opposite-sign operands near the smallest normal is #NUM! (calipers formulas_math_basic I88, H89)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
MOD of two tiny opposite-sign operands near the smallest normal double gives Excel's `#NUM!` (or whatever the probe shows)
where the engine returns a number. The rule is fitted from oracle evidence, and the cells in the calipers goldens below match.

## Context
Found by the M5 planner run of 2026-10-25 (calipers fetched, `rustsheet-eval run --suite calipers`, HEAD of
origin/main). No open task names these cells. T-2654 (MOD's huge opposite-sign divisor) and T-2661 (POWER) fixed other
cells of the same sheets. Neither covers these.

In `calipers/roundtrip/formulas_math_basic/golden.xlsx` (sheet `Math Tests`) and its twin
`calipers/roundtrip/formulas_datetime/golden.xlsx` (sheet `Sheet2`), C10 is `=10^(-307)` (1.0000000000000001E-307) and
C11 is `=-10^(-307)` (-9.999999999999995E-308; the two are not exact negatives). 4 cells, visibly off (wrong type):

| cell | formula | Excel (desktop golden) | ours |
|---|---|---|---|
| I88 (both books) | `MOD($C$10,$C$11)` | `#NUM!` | -9.99999999999999E-308 |
| H89 (both books) | `MOD($C$11,$C$10)` | `#NUM!` | 6E-323 |

Controls in the same rows already match. `MOD($C$10,$C$10)` and `MOD($C$11,$C$11)` are 0. `MOD($C$10,$C$7)` (d = -1) is
0 (T-2654's rule). `MOD($C$11,$C$13)` is C11. `MOD(-42.5,$C$10)` is `#NUM!` (huge quotient, numbers.md "MOD is #NUM! for a
huge quotient").

The quotient here is about -1.0000000000000007 (`$C$10/$C$11` in I62), so the huge-quotient rule does not explain the
error. Both failing cells pass through a subnormal: in I88, `fmod` is 1E-307 - 9.999999999999995E-308, about 6E-323,
before the shift by d; in H89, the shifted result `fmod + d` is that subnormal. Hypotheses to separate:
- (a) a subnormal remainder, before or after the shift, is `#NUM!`;
- (b) only a subnormal final result is `#NUM!`, and I88 comes from something else, such as a check of `n - d*INT(n/d)`
  against `|d|`;
- (c) a quotient-based path `n - d*INT(n/d)` whose product or difference underflows is `#NUM!`.

The flush rules of numbers.md "Subnormal results become 0" (`*` and `/` flush, `+` and `-` keep the subnormal) do not
predict `#NUM!` here, so this is a separate quirk.

Code: `excel_mod` in crates/rustsheet-engine/src/functions/util.rs, and `fn_mod` in
functions/math_and_trigonometry/mathematical.rs. The golden is desktop evidence, so probe first with excel-graph
(`excel-probe` skill). If Excel for the web disagrees with the golden, record a desktop-only rule and file an M14
follow-up instead of matching the golden (decision 0039).

## Acceptance criteria
- [x] A probes case tagged M5 (excel-graph) separates hypotheses (a)-(c). It covers `MOD(n,d)` for opposite-sign pairs
      whose fmod or shifted result is subnormal, normal or exactly 0: n and d among ±2^-1022, ±1E-307, ±1.5E-307,
      ±2.2250738585072014E-308, ±1E-300 with d = -(n·(1±k·2^-52)) for k = 1, 2, 4; plus the two calipers pairs and
      same-sign controls. Each pair has an `ISERROR` or `(x)=0` companion where Graph's 0 for a kept subnormal would
      hide the answer (decision 0042).
- [x] The engine matches every cell of the probe. (The calipers.txt part waits on T-0463's ratchet; see Outcome.) The calipers I88 and H89 cells of both workbooks pass
      (`rustsheet-eval run --suite calipers` after `rustsheet-eval fetch --suite calipers`), and their lines leave
      eval/baseline/calipers.txt (`rustsheet-eval update-baseline --suite calipers`) with no new known failure.
- [x] numbers.md gets an entry "MOD near the smallest normal" with the fitted rule, the evidence and `Status: VERIFIED`.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
`semantics`: the owned cells are a wrong type (`#NUM!` against a number). The task is done when every owned cell has
Excel's type and value.

## Out of scope
- CEILING.MATH and FLOOR.MATH on the same inputs (T-2715).
- MOD's huge-quotient threshold and the huge opposite-sign divisor (done, T-0141 and T-2654).

## Outcome
- Probes (excel-graph, 2026-10-25, all tagged M5 except the snap case, M2): t2714-mod-near-smallest-normal (131 pairs, both
  orders), -b (70), -c (66 pairs, both orders) and t2714-snap-tiny-addends (45 pairs). Hypothesis (a) holds, with one
  arm none of (a)-(c) predicted: a subnormal fmod or a subnormal shift is `#NUM!`, but when both are subnormal and |d| is
  exactly 2^-1022 the result is `d - fmod`. All 464 MOD cells fit (numbers.md "MOD near the smallest normal").
- Found in passing: the near-zero snap's 8-ULP bound is flushed when subnormal, so addends below 2^-973 never snap
  (`1E-300-1E-300*(1+2^-52)` keeps its subnormal). That is fixed here too (`snap_near_zero`), because the MOD probe's
  `A+B` column depends on it.
- calipers: I88 and H89 pass in both workbooks. The same workbooks' I23 and H24 (`=$C$10+$C$11`) keep the
  subnormal now, which desktop Excel saved as 0. They pass through a new calipers `cached-overrides.tsv` (decisions
  0032, 0090) pointing at the probe's identical rows 130 and 131.
- Eval delta: probes +4 cases (3,051 cells, all passing), known failures unchanged in every gated suite, 0 regressions.
  calipers (not gated): the 8 owned cells pass. `update-baseline --suite calipers` still refuses because of 36 cells
  (pv_goldens PPMT/NPV ulps and CHISQ.INV) that fail on `main` too (checked with a build of `main`; owned by T-2708 and
  T-0463), so calipers.txt is not ratcheted here: the four I88 and H89 `F` lines leave it with T-0463's ratchet.
