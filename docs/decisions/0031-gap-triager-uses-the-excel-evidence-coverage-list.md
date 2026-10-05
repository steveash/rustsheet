# 0031 — Gap-triager uses the Excel-evidence coverage list

- **Status:** accepted (2026-10-05, T-0020)
- **Context:** A function's pass rate on 3 cells says little. The 2026-10-04 eval had 435 functions
  with checked cells, 112 of them with fewer than 10, and about 60 implemented functions with none.
  The gap-triager filed tasks only from failing cells, so functions that are thinly tested, or
  not tested at all, never got a task. `make eval` now writes "Excel evidence per function" to
  report.md (and `evidence`/`unevidenced` to report.json). It counts only cells whose
  expectations come from Excel (oracle sidecars or Excel-cached values), and flags functions below
  `EVIDENCE_THRESHOLD` (25 cells).
- **Decision:** The gap-triager agent (`.claude/agents/gap-triager.md`, a protected path) also
  reads that section. It files probe or generator tasks for in-scope functions (by catalog
  milestone) that have fewer than 25 Excel-evidence cells or none at all. The steward copies the
  section's one-line summary into the docs/status.md snapshot.
- **Consequences:** Evidence gaps become tasks, not only failures. The evidence report is
  informational and never part of the eval gate, so no threshold is enforced and no baseline changes.
  Functions with no Excel evidence are treated as unverified, even when they pass hand-written cases.
