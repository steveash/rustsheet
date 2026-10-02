# Status

Updated by the steward routine (daily) and by workers (one line per merged task).

## Snapshot — 2026-10-01 (M0 bootstrap)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 207 | 71,387 | 97.33% | 1,909 |
| smoke | 3 | 23 | 100% | 0 |
| probes | 2 | pending oracle | — | — |
| gen-m2 | 3 | pending oracle | — | — |

Hygiene: lint-debt 99 allow entries (engine 59, xlsx 37, bindings 3) · duplicated lines
9,153 · oversized files 21.

**Critical lane:** harness, api, engine

(Ordered list read by the critical-lane worker; harness first for oracle bring-up, then the M1 API/engine work.)

## Needs owner
- Excel oracle credentials: owner has a Microsoft 365 Business tenant (Graph-compatible). Pending:
  dedicated SharePoint site + app registration (`Sites.Selected`) + four `RUSTSHEET_ORACLE_*`
  environment variables (step-by-step guide in `docs/architecture/oracle.md`). Unblocks T-0004 and its dependents.
- Routines are enabled with the repo attached, but their prompts are the pre-review drafts: replace each
  routine's prompt with the current text in `docs/process/routines.md`.

## Log
- 2026-10-02 — Owner set `main` as the GitHub default branch and enabled the routines (repo attached).
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0002: pinned external suite fetcher (`rustsheet-eval fetch`); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0201: criteria/wildcard matching consolidated in functions/criteria.rs; fixed COUNTIF "<>text" on non-text cells; eval 0 regressions, 0 baseline fixes (+2 new spec cases pass).
- 2026-10-02 — T-0003: suite `poi` (Apache POI, fetched, pinned); 334 cases, 3,526 cells, 82.25%; baseline +633 known failures (new suite, 0 regressions); filed T-0111 (timeouts), T-0112 (imports).
- 2026-10-02 — T-0204: coercing range walk shared by VARA/STDEVA family and GCD/LCM; duplicated lines 8829→8610; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0007: pending cases now run crash-only (panic/timeout/error reported, per-suite crashes column); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0205: D-function criteria routed through shared criteria grammar (CriteriaFlavor::Database); eval 0 regressions, 0 fixed (+1 new smoke case passes).
