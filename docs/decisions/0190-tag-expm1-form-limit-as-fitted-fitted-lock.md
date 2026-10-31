# 0190 — Tag EXPM1_FORM_LIMIT as FITTED (fitted lock 80 to 81)

- **Status:** accepted (2026-10-31, T-3385). It applies decision 0188 R4.3 to one constant that
  T-3361's tagging pass left out.
- **Context:** `math_and_trigonometry/hyperbolic.rs` `EXPM1_FORM_LIMIT` (1.0) chooses where SINH and
  TANH switch from the expm1 forms to the exp forms (T-2134). T-3385 checked whether that cut
  follows from the math. It does not, because Excel's cells place it. In
  probes-numeric/t2178-hyperbolic-unit-sweep the exp forms miss `TANH(0.995054655025377)` and
  `TANH(±0.9950546550253772)`, where the expm1 forms match. On the 20 control points in (1, 1.5)
  the expm1 forms miss from `SINH(1.0375)`, where the exp forms match all 40 cells. Excel's switch
  therefore lies in 0.99505..1.0375 (docs/excel-semantics/numbers.md "SINH and TANH below 1 are
  built from expm1").
- **Decision:** the constant carries `FITTED(T-2134, 2070 cells, SINH and TANH |x| 1E-10..1.49; cut
  bracketed 0.99505..1.0375)`. quality/fitted.json is raised for hyperbolic.rs from 2 to 3 with
  `scripts/check_fitted.py --update`. The ratchet says the count only falls; this raise records an
  existing fit and adds no new one. Neither the value nor any code path changes, so R4.1's hold-out
  and R4.2's fresh sweep owe nothing: every cut inside the bracket scores the same on the oracle
  cells.
- **Consequences:** the tag now makes R4.3's range check visible for this cut. The fitted ratchet
  stays a one-way gate for new fits. Raising the count for an existing, untagged fit still needs a
  decision record (this one is the pattern). Narrowing the bracket needs a probe of inputs where the
  two forms disagree (T-3395).
