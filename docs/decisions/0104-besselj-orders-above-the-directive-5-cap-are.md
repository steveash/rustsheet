# 0104 — BESSELJ orders above the directive-5 cap are accepted probe deviations

- **Status:** accepted (2026-10-23, T-2139)
- **Context:** probes/m9-bessel-order-limits (T-2139) shows Excel's own BESSELJ order cap is
  10,000,000: `BESSELJ(10,n)` is 0 up to it, and `BESSELJ(1E5,100001)` is 0.0094462635180178.
  The engine refuses every Bessel order above 100,000 (`MAX_ORDER` in
  `functions/engineering/transcendental/bessel_excel.rs`) to bound per-cell work on untrusted
  input, CLAUDE.md directive 5; engineering.md "BESSEL order cap" already records that as a
  DEVIATION under decision 0014. Seven of the new probe cells fall between the two caps, so the
  engine gives `#NUM!` where Excel gives a value. Adding them to `eval/suites/probes/deviations`
  touches a protected path (decision 0036), which needs a record.
- **Decision:** Those seven cells (`Calc!B270`, `B275`, `B277`, `B278`, `D427`, `D428`, `D433`)
  are listed in the probes suite's `deviations` file, pinned to `#NUM!` and citing decision 0014
  and "BESSEL order cap". The cap stays at 100,000 (raising it is out of T-2139's scope).
- **Consequences:** The cells are excluded from bit-exact scoring and any other value for them
  fails the gate. If `MAX_ORDER` is ever raised, the lines go stale and are removed in that PR,
  and the tiny-x quirk (`BESSELJ(1E-300,100001)` is `#NUM!` in Excel) must then be matched
  separately. No change for other functions or gates.
