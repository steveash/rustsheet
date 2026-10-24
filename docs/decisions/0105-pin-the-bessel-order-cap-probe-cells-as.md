# 0105 — Pin the BESSEL order-cap probe cells as deviations (T-2421)

- **Status:** accepted (2026-10-24, T-2421)
- **Context:** Decision 0014 caps the BESSEL order at 100,000 to bound per-cell work on untrusted input (CLAUDE.md directive 5). Excel gives 0 for `BESSELI(2.5,100001)` and `BESSELI(2.5,1E+6)`, ours `#NUM!`. T-2421's acceptance criteria require the DEVIATION entry "BESSEL order cap" (engineering.md) to cite a committed probe, so probes/eng-besselk-underflow-bit-shift holds both cells with excel-graph values. A cell that differs by design must be listed in the suite's `deviations` file (decision 0036), a protected path.
- **Decision:** `eval/suites/probes/deviations` lists the two cells (`S!A179`, `S!A180`) with decision 0014 and the `#bessel-order-cap` anchor, pinned at `#NUM!`. No other cell is added and no gate is relaxed.
- **Consequences:** The probe passes with two accepted deviations; a different value in either cell fails the gate like any regression. The cap itself is unchanged.
