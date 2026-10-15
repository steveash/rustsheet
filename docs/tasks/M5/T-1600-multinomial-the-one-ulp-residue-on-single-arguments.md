---
id: T-1600
title: "MULTINOMIAL: the one-ulp residue on single arguments, and numeric text after a negative"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
MULTINOMIAL is bit-exact on the 8 probed cells T-1190's fit leaves one ulp off, and numeric text after a negative is evidenced.

## Context
T-1190 fitted Excel's MULTINOMIAL as a log10 running sum (`multinomial_value`, functions/math_and_trigonometry/multinomial.rs;
docs/excel-semantics/numbers.md "MULTINOMIAL is inexact"): 64/72 bit-exact, 72/72 within 1 ulp. The misses are all single
arguments, alone or padded with zeros: probes/multinomial-algorithm-sweep A5 `(5)`, A8 `(8)`, A9 `(10)`, A14 `(30)`, A15 `(40)`,
A21 `(140)`, A57 `(0,5)`, A58 `(5,0)`. Hypotheses to try: a lone nonzero argument takes a different path (zeros skipped first),
or the final step is not `exp(acc * ln 10)` for |acc| near 0. Probe more single arguments (every n from 1 to 170) before fitting
(decision 0056). Also unprobed: a numeric-text range cell after a negative (a range holding -1, "2", 3: the engine now gives
`#NUM!`; before T-1190 it was `#VALUE!`), and formatted numeric text in a range cell (" 2", "1e1", "50%", date text).
Note: the last step uses the platform `exp`, which may differ by an ulp off glibc.

## Acceptance criteria
- [ ] A probe of all single arguments 1..170 and the coercion cells above, filled by excel-graph; the candidates and their scores recorded.
- [ ] The 8 cells pass, or the task is blocked as time-boxed (decision 0056) with the candidates listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COMBIN (T-1598).
