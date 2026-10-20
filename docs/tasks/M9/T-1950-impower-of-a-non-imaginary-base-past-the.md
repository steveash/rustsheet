---
id: T-1950
title: "IMPOWER of a non-imaginary base past the angle cut gives values near 1.797E+308, not #NUM!"
milestone: M9
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMPOWER matches Excel for a base that is not a pure imaginary number once the angle `atan2(y,x)·n` passes the cut (134217727).

## Context
probes/eng-impower-angle-cut: `IMPOWER("0.7071067811865476+0.7071067811865476i",170891318)` is `1.79769293021926E+308+1.79769293021926E+308i` in Excel and `…,170891320` is `…925E+308` (parts differ, so not a clamp); ours is `#NUM!` (`fn_impower` in functions/engineering/complex.rs). Needs a model of Excel's out-of-range sine/cosine (excel-kernel-playbook) and probes that tell pure imaginary (`#NUM!`) from the rest. See engineering.md "T-1625 probes". Lead for the modulus difference (fidelity review of T-1892, not yet probed): the literal reads as the 15-digit 0.707106781186547, and from it `sqrt(x*x+y*y)` gives 1-7·2^-53 while `hypot(x,y)` gives 1-6·2^-53, Excel's per-factor value; probe `IMABS` of that string to confirm.

## Acceptance criteria
- [x] All eleven failing cells A9-A19 of eng-impower-angle-cut (the `0.7071067811865476+0.7071067811865476i` rows; A9-A17 are inside the cut and differ from Excel in the printed digits, so the modulus power `|z|^n` of a base near radius 1 also differs) pass, with more probe cells for the rule that picks `#NUM!` or a huge value.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/eng-impower-past-cut, 19 cells, excel-graph): past IMPOWER's angle cut Excel's sine and cosine both
read as DBL_MAX in every quadrant, so each part is `|z|^n·DBL_MAX` (two equal positive parts), and `#NUM!` once that
is not below DBL_MAX (a radius of 1 or more). Excel's modulus is `hypot`, not `sqrt(x²+y²)` (IMABS of the 16-digit
1/√2 literal is 1-6·2^-53). `fn_impower` and IMABS use one `modulus` helper; the past-cut branch replaces the old
`#NUM!` (and its TODO). engineering.md: new entry "IMPOWER past the angle cut"; the two UNVERIFIED bullets resolved.
Eval: probes known failures -11 (eng-impower-angle-cut A9-A19), +19 new passing cells, no regressions.
