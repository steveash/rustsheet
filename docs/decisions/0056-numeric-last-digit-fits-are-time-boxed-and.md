# 0056 — Numeric last-digit fits are time-boxed and kernel-first

- **Status:** accepted (2026-10-14, owner decision). It extends decision 0026 (fit only against an
  oracle sweep) and keeps decision 0052 (accepted deviations are directive-5 limits only).
- **Context:** About 9,400 function-cell pairs are numeric near misses (2026-10-14 eval). Fit tasks
  are split per function, although a few kernels feed most of them. Per-function fitting has also
  sunk time without a result: GEOMEAN's one-ulp residue has taken about ten tasks (T-1076, T-1099,
  T-1123, T-1284, ...) and is still blocked.
- **Decision:**
  - The numeric track runs alongside M2, owner request 2026-10-14. T-1522 to T-1525 are P1 for
    the opus tier, although their milestone is M9.
  - Kernel first: fit work follows the T-1522 kernel map, highest cell count first. A fix lands at
    the kernel, so every function using it gains.
  - Every fit is a fit-lab comparison (T-1523) of named candidates against an oracle sweep. The
    task records each candidate's bit-exact and within-4-ulp counts, including the rejected ones.
  - Time-box: if two attempts (two sessions) do not raise the bit-exact count on the sweep, the
    task is set to `blocked` with the candidates tried. It reopens only with a new hypothesis:
    a cross-kernel finding such as the x87 model (T-1524), or new evidence.
  - Residue within 4 ulp counts against Phase 1's 0.1% bit-exact allowance. It is never a
    DEVIATION (0052).
- **Consequences:** fewer, larger numeric tasks, each scored against data. A stuck function stops
  absorbing sessions. M9's exit criteria are unchanged.
