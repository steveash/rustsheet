# 0056 — Numeric last-digit fits are time-boxed and kernel-first

- **Status:** accepted (2026-10-14, owner decision). It amends decision 0026's milestone order for
  this track (owner-directed work only; `tasks.py next` and decision 0023 are unchanged), extends
  0026's "fit only against an oracle sweep" rule, and keeps decision 0052 (accepted deviations are directive-5 limits only).
- **Context:** About 9,400 function-cell pairs are numeric near misses (2026-10-14 eval). Fit tasks
  are split per function, although a few kernels feed most of them. Per-function fitting has also
  sunk time without a result: GEOMEAN's one-ulp residue has taken about ten tasks (T-1076, T-1099,
  T-1123, T-1284, ...) and is still blocked.
- **Decision:**
  - The numeric track runs alongside M2, owner request 2026-10-14. The owner-directed session
    works T-1522 to T-1525 now (P1, opus tier, milestone M9). The automatic pick order is
    unchanged (`tasks.py next`, decisions 0023 and 0026), so routine workers keep picking M2
    first. The kernel fit tasks follow the T-1522 map in owner-directed sessions, and through
    the normal order after M2.
  - Kernel first: fit work follows the T-1522 kernel map, highest cell count first. A fix lands at
    the kernel, so every function using it gains.
  - Every fit is a fit-lab comparison (T-1523) of named candidates against an oracle sweep. The
    task records each candidate's bit-exact and within-4-ulp counts, including the rejected ones.
  - Time-box: an attempt is one session that runs the fit lab on the kernel's sweep and records
    the result in the task. If two attempts do not raise the bit-exact count, the task gets
    `status: blocked` and a section "Blocked because: time-boxed (decision 0056)" listing the
    candidates tried. Unlike other blocked tasks, the steward does not reopen it when a
    dependency lands. It reopens only when a new hypothesis is recorded in that section: a
    cross-kernel finding such as the x87 model (T-1524), or new oracle evidence.
  - Residue within 4 ulp counts against Phase 1's 0.1% bit-exact allowance. It is never a
    DEVIATION (0052).
- **Consequences:** fewer, larger numeric tasks, each scored against data. A stuck function stops
  absorbing sessions. M9's exit criteria are unchanged.
