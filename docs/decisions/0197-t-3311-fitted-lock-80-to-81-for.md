# 0197 — T-3311: fitted lock 80 to 81 for NEWTON_START_MAX, duplication banked

- **Status:** accepted (2026-10-31, T-3311). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3311 makes Excel's lower-tail gamma quantile at a shape above 1 the Newton steps from the
  leading-factor root on T-3060's cdf form, keeping the first iterate whose cdf is nearest p
  (`incomplete_gamma::closed_form::excel_newton_quantile`; docs/excel-semantics/statistics.md, "Deep lower tail at a
  shape above 1"). The steps apply only where the form at the root is within `NEWTON_START_MAX` of p. That cut is a
  region chosen by scoring oracle cells, so it is a fit (decision 0188 R4):
  - it carries a `FITTED(T-3311, ...)` tag and applies only in its measured range;
  - its hold-out, fresh sweeps and Gate A envelope are in T-3311's Outcome, with the 50-digit references and the
    scripts in docs/research/t3311.
- **Decision:**
  - quality/fitted.json rises by one, for this one new fit (`scripts/check_fitted.py --update`).
  - quality/duplication.json banks a drop in duplicated lines (`scripts/check_duplication.py --update`; the count
    only fell).
- **Consequences:** none for the gates. The fitted ratchet stays one-way for new fits, each with its own record.
