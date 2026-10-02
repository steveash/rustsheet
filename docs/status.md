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
  app registration + `RUSTSHEET_ORACLE_*` environment variables / GitHub `oracle` environment
  secrets (`docs/architecture/oracle.md`). Unblocks T-0004 and its dependents.
- Set `main` as the GitHub default branch (repo Settings → General → Default branch).
- Create the three routines from `docs/process/routines.md` in the claude.ai routines UI (repo + GitHub attached).

## Log
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
