---
id: T-3627
title: "Probe ROUNDUP of a huge value at a huge negative num_digits (6.44E+265 at -43376 is the value, not #NUM!)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Find the rule that makes `ROUNDUP(6.4421516092447E+265,-43376)` return its argument unchanged in Excel, record it in
docs/excel-semantics/numbers.md ("ROUNDUP of a huge value at a huge negative num_digits", today an observation), and
file the engine task if the engine differs.

## Context
- Found by T-3555's first Gate A draw for ROUNDUP (a batch replaced before commit, so the point is not in
  `eval/accuracy/`): excel-graph, 2026-11-01, gave 6.4421516092447E+265. numbers.md "ROUND family: huge num_digits wrap"
  says ROUNDUP keeps the sign of `num_digits` and takes its magnitude modulo 65,536, which gives -43376, so the unit
  10^43376 overflows and the answer should be `#NUM!`. The engine (`effective_digits` in
  `crates/rustsheet-engine/src/decimal_round.rs`) follows that rule.
- The same draw had `ROUNDUP(-3.41521682833705E-207,-11541)`, `ROUNDUP(-2.33253918200575E+106,-678)` and
  `ROUNDUP(7712487.6865209,-400)` as `#NUM!`, and `ROUNDUP(4.72880001004526E+149,22178)` as the value. Every earlier
  probe of a huge negative `num_digits` used a small value (123.456, 0.5, 42, 1E+15; probes/round-huge-digits*,
  probes/roundup-wrapped-scaling), and `ROUNDUP(123.456,-65535)` is `#NUM!` there.
- Committed evidence for the same value: `eval/accuracy/tables/ROUNDUP/s1` Calc!A111, `ROUNDUP(6.4421516092447E+265,-320)`,
  is `#NUM!`. The 16-bit kept-digit count below fits every cell named here (-54 at -320; -43,110, which wraps to +22,426, at
  -43,376), but `ROUNDUP(123.456,-65535)` (count -65,532, which wraps to +4) does not, so the count alone is not the rule.
- Candidates to separate: a 16-bit count of the kept digits (`num_digits` + exponent + 1, as ROUND counts in 32 bits);
  a threshold at |`num_digits`| > 32,767; a dependence on the value's exponent. A probe grid of exponents (-300, -10, 0, 10,
  100, 265, 307) × `num_digits` (-32767, -32768, -32769, -43376, -65535 and a few between) for ROUNDUP and ROUNDDOWN
  (and ROUND as a control) should separate them. Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe case with oracle values separates the candidates; numbers.md states the rule (VERIFIED) or the narrowed
      observation, citing the cells.
- [x] If the engine differs, an engine task is filed with the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Gate A tables: T-3555's tables draw `num_digits` within ±400 only.

## Outcome
- Rule found (VERIFIED, excel-graph, 2026-11-01): ROUNDUP and ROUNDDOWN narrow `num_digits` to k as before. When
  |k| >= 32,768, which no longer fits a signed 16-bit integer, they round the 15-digit cut at 100 places: at 10^-100
  for positive k and at 10^100 for negative k. ROUNDUP's unit 10^-k, added as a double, underflows to 0 for positive k,
  so ROUNDUP rounds down. For negative k it overflows, so ROUNDUP is `#NUM!` whenever a digit is dropped. The rule
  explains the T-3555 cell: `ROUNDUP(6.4421516092447E+265,-43376)` has no digit below 10^100, so nothing is added.
  numbers.md, "ROUNDUP and ROUNDDOWN round at 100 places past a 16-bit num_digits", replaces the UNVERIFIED observation.
- How the candidates were separated:
  - **A 16-bit kept-digit count** is ruled out. `ROUNDDOWN(1.23456789012345E-95,65535)` keeps 6 digits, which no count
    wrap gives, and the step is at 10^∓100 for every k from 32,768 to 65,535.
  - **A threshold at |k| > 32,767** is ruled in: 32,767 is plain and 32,768 is not.
  - **The value's exponent** matters only through the fixed 100-place rounding.
- Probes (probes/, M2, excel-graph):
  - t3627-roundup-rounddown-digits-by-exponent: 620 formula cells. x = 1.5E-300 to 1.5E+300 in steps of 10^20,
    k = ±400, ±1000, ±32767, ±32768, ±65535, for ROUNDUP and ROUNDDOWN.
  - t3627-roundup-rounddown-huge-digits-place: 322 formula cells. x = 1.23456789012345E∓90 to E∓112, k = ±32768,
    ±40000, ±65535 and 98304.
  - A model of the rule, the Gate A ROUNDUP/ROUNDDOWN math with the 100-place step added, matches all 942 cells. It
    also matches every literal-argument ROUNDUP/ROUNDDOWN cell of probes/round-huge-digits*, which still pass.
  - ROUND was not re-probed: every ROUND cell already follows the 32-bit rule. TRUNC was not probed past 32,767.
- Engine: it differs (it rounds at k itself), so 309 of the 942 cells fail. They are baselined as F lines in
  eval/baseline/probes.txt and owned by **T-3635** (functions-math, sonnet, S), filed here.
- Gate A is unchanged: T-3555's definitions draw `num_digits` within ±400 and give no math value past it.
- Eval delta: 2 new probe cases with 942 cells, 633 passing; known failures +309, all owned by T-3635. No regressions.
