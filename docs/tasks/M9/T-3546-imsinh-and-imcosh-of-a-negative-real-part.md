---
id: T-3546
title: "IMSINH and IMCOSH of a negative real part: pin where the log form starts (below |x| = 708.94)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMSINH and IMCOSH of x + yi with x < 0 switch to the log form (complex/trig.rs `hyperbolic_log_form`) where Excel
does, which is below |x| = 708.94, not at |x| > 709 as for x > 0; and the log form's digits for x < 0 match.

## Context
- Found by T-3525's log-band sweep (probes-numeric/t3525-im-hyperbolic-log-band-sweep, excel-graph): with x < 0,
  `IMSINH("-708.993723853194-0.284042075249253i")` and `IMCOSH` of it are `#NUM!` (sin y < 0: the log form's
  signature), `"-708.952568260886+1.74023049176272i"` too (cos y < 0), and `IMSINH("-708.931415280617+0.631619003955773i")`
  is -3.09659439784082E+307+2.26546023028534E+307i, the log form's digits (product form: ...096E+307+...528E+307i).
  For x > 0, `"708.959727…+1.343…i"` keeps the product form and the T-2231 bracket (709, 709.0000000001] stands.
  8 of the sweep's 10 engine misses are these; the other 2 are `IMSINH`/`IMCOSH("-709.183834084079+1.0222540398548i")`,
  one 15th-digit miss of the log form's imaginary part at x < 0.
- The rule may depend on y, not on a single |x| cut: S!A84 (x = -708.931) has the log form's digits, S!A38
  (x = -708.9937) is `#NUM!`, while S!A71 (x = -708.9941, both factors positive) is a number that differs from the
  math in the 13th digit (docs/research/t3525/gate_a.tsv). Design the probe to separate x from y.
- One hypothesis to probe (not evidence): the negative side cuts where EXP(x) leaves the normal range
  (ln DBL_MIN = -708.3964).
- docs/excel-semantics/engineering.md "IMCOSH and IMSINH past |x| = 709 work in log space (T-2231)" has the notes.

## Acceptance criteria
- [ ] A probe brackets the negative-side cut to 1e-10 (excel-graph), and the cut is implemented from that bracket
      (an evidenced constant, not a fit), with engineering.md updated.
- [ ] The 8 cut cells of the t3525 log-band sweep pass; no older IMSINH/IMCOSH cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1/u band (T-3525 keeps it).
