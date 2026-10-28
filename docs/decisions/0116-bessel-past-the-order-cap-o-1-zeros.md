# 0116 — BESSEL past the order cap: O(1) zeros retire stale probe deviations and pin the rest

- **Status:** accepted (2026-10-25, T-2546)
- **Context:** T-2546 makes BESSELI, BESSELJ and BESSELK above the directive-5 order cap
  (`MAX_ORDER`, 100,000) return Excel's 0 where an O(1) bound proves Excel's recurrence ends in
  exactly 0, still without running a recurrence (engineering.md "BESSEL order cap",
  probes/t2546-bessel-past-cap). Eight lines of `eval/suites/probes/deviations` (decisions 0014,
  0104) now match Excel and are stale: m9-bessel-order-limits `Calc!B270`, `B275`, `B277`, `D427`,
  `D428`, `D433` and eng-besselk-underflow-bit-shift `S!A179`, `S!A180`. Six cells of the new probe
  still differ by design, as the remaining DEVIATION: `BESSELJ` with x near or above n (`Calc!A1`,
  `A24`) and `BESSELY` with a large x (`Calc!D1:D4`), where Excel has a value and the engine gives
  `#NUM!` rather than run 10^5 to 10^7 recurrence steps. Editing the deviations file is a protected
  path (decision 0036).
- **Decision:** Remove the eight stale lines and list the six cells in the probes suite's
  `deviations` file, pinned to `#NUM!`, citing decision 0014 and "BESSEL order cap". The order cap
  stays at 100,000; `m9-bessel-order-limits` `Calc!B278` (`BESSELJ(1E5,100001)`) stays listed.
- **Consequences:** The six cells are excluded from bit-exact scoring instead of being known
  failures, and any other value for them fails the gate. A later task that computes BESSELJ near
  x = n or BESSELY at large x past the cap in bounded work removes the lines it fixes. No change
  for other functions or gates.
