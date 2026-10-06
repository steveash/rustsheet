---
id: T-0174
title: "Probe the tiny-magnitude cut-offs of number text (5e-310, 2e-308, 9.9e-308, 0e-400) for T-0154"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence that pins the two boundaries T-0154 needs: below which magnitude number text is
`#VALUE!`, and up to which magnitude it reads as 0 instead of the (subnormal) value, so T-0154 can be
implemented without fitting a boundary by guess.

## Context
T-0920 found (probes/text-number-words, excel-graph 2026-10-05; numbers.md "Number text of 1E+308 or
more is not a number") that `0+"1E-309"` is 0 while `0+"1E-310"` is `#VALUE!`, and that zero
mantissas with a large exponent (`"0e309"`) are `#VALUE!`. T-0154 (engine, todo) lists the open
questions: the exact lower cut-off between 1E-309 and 1E-310, and whether the flush-to-0 range ends
at the smallest normal double (2.2250738585072014E-308) or at 1E-307. Splitting the probe out keeps
T-0154 a pure implementation task.

Probe case `eval/suites/probes/cases/text-number-tiny.case.json` (excel-probe skill). Each input
text in column A (as a text cell, not a formula literal), with `=0+A1` in B, `=VALUE(A1)` in C and
`=ISERROR(0+A1)` in D:
- cut-off: `5e-310`, `9.99e-310`, `1.000001e-309`, `9.9999e-310`, `2e-309`, `1.5e-309`;
- flush range: `2e-308`, `2.2250738585072e-308`, `2.2250738585073e-308`, `9.9e-308`, `1e-307`, `1.1e-307`;
- zero mantissas: `0e-400`, `0e308`, `0e309`, `0.0e309`, `-0e309`;
- signs: `-1e-309`, `-1e-310`, `-2e-308`.
If Excel shows a monotone cut-off, add a bisection round (a second case) until the boundary is known
to 3 significant digits; record each round's inputs.

## Acceptance criteria
- [ ] Probe case(s) and oracle sidecar(s) committed under eval/suites/probes/cases/.
- [ ] numbers.md states both boundaries (cut-off to 3 significant digits, flush range end) and the
      zero-mantissa rule, VERIFIED with the probe cells as evidence.
- [ ] Engine mismatches in the new case(s) are known failures in eval/baseline/probes.txt (new case,
      not a regression) and are named in T-0154's Context, which implements them; no engine change here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `parse_excel_number` (T-0154).
- Formula number literals (T-0144) and huge magnitudes (already VERIFIED).
