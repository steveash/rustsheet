---
id: T-3627
title: "Probe ROUNDUP of a huge value at a huge negative num_digits (6.44E+265 at -43376 is the value, not #NUM!)"
milestone: M2
lane: research
status: todo
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
- Candidates to separate: a 16-bit count of the kept digits (`num_digits` + exponent + 1, as ROUND counts in 32 bits);
  a threshold at |`num_digits`| > 32,767; a dependence on the value's exponent. A probe grid of exponents (-300, -10, 0, 10,
  100, 265, 307) × `num_digits` (-32767, -32768, -32769, -43376, -65535 and a few between) for ROUNDUP and ROUNDDOWN
  (and ROUND as a control) should separate them. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values separates the candidates; numbers.md states the rule (VERIFIED) or the narrowed
      observation, citing the cells.
- [ ] If the engine differs, an engine task is filed with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Gate A tables: T-3555's tables draw `num_digits` within ±400 only.
