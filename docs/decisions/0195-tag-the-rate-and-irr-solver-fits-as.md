# 0195 — Tag the RATE and IRR solver fits as FITTED (fitted lock 81 to 86)

- **Status:** accepted (2026-10-31, T-3395). It applies decision 0188 R4.3 to five existing fits in
  `functions/financial/financial_util.rs` that T-3361's tagging pass left out, as decision 0190 did
  for `EXPM1_FORM_LIMIT`.
- **Context:** T-3395 classified the RATE and IRR solver constants by reviewer-excel-fidelity check
  7. Five were chosen on Excel cells:
  - RATE's start offset 0.001: scored against 1E-4 and -0.001 by the fit lab's `rate_candidates`
    (T-2005); the model is bit-exact on all 782 cells of the fit lab's `rate` set.
  - RATE's residual stop |f| < 1E-7 (the same `TOLERANCE` as its documented step stop): scored
    against 1E-6 and 1E-8 (T-2005; financial.md "RATE is a secant on the FV residual").
  - RATE's cap-return tolerance 1E-4: one value placed inside the brackets the three cap probes
    give for the step, |f| and 1 + r tests (T-2689; probes/rate-cap-return-rule, rate-pmt-zero-cap,
    rate-step-cap, 178 RATE cells).
  - IRR's start offset 1E-3 and residual stop 1E-7: chosen by scoring on the 263-cell fit set
    (T-1533; financial.md "IRR is a secant in d = 1 - v": +1E-3 on both sides scores 121, a 1E-6 or
    1E-8 residual 188 or 180).
  - `IRR_FAR_ROOT_BOUND` 1E+7: a round value placed between Excel's 9999999.994 (returns) and 1E+7
    (`#NUM!`) on 35 probe cells (T-2727).
  Not fits, so not tagged, and their doc comments now say why: RATE's and IRR's 1E-7 step stops
  (Microsoft's documented 0.0000001 and 0.00001 percent), `RATE_MAX_STEPS` (pinned exactly by
  probes/rate-step-cap), `IRR_GUESS_BOUND` (pinned exactly at 2^53) and the IRR and XIRR loop caps
  (directive-5 limits, financial.md "The IRR and XIRR solver caps").
- **Decision:** the five carry `FITTED(...)` tags with the cell counts and input ranges read from
  their fit sets (the fit lab's `rate` and `irr` sweeps, the probe cases). quality/fitted.json is
  raised for financial_util.rs from 3 to 8 with `scripts/check_fitted.py --update`. No value or code
  path changes, so R4.1's hold-out and R4.2's fresh sweep owe nothing new.
- **Consequences:** R4.3's range check is visible for these fits: RATE's offset and IRR's secant
  constants apply to every call, past the ranges their tags name, as does f_density.rs's small-shape
  constant below `LARGE_DF_MIN`; measuring or bounding them is T-3478. The ratchet stays one-way
  for new fits; tagging an existing, untagged fit still needs a decision record (decisions 0190 and this one are the pattern).
