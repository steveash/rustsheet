# 0002 — Strict bit-exact scoring with a known-failures ratchet

- **Status:** accepted (2026-10-01)
- **Context:** IronCalc's own harness used a 5e-8 relative tolerance. That hid 1,909 cells that
  differ from Excel. Our goal is parity, not "close".
- **Decision:**
  - A cell passes only if it is bit-identical to Excel. Near misses are classified but count as
    failures.
  - CI gates on a *known-failures* baseline per suite: no previously passing cell may fail, and
    the baseline must be regenerated (never hand-edited) when cells are fixed or cases are added.
- **Consequences:**
  - Precision work is visible and prioritized.
  - Baseline files are small (failures only) and double as the to-do list.
  - Allowing a regression requires `--allow-regressions` plus a decision record.
