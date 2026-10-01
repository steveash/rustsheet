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

**Critical lane:** harness (oracle bring-up), then api/engine for M1.

## Needs owner
- Excel oracle credentials: owner has a Microsoft 365 Business tenant (Graph-compatible). Pending:
  app registration + `RUSTSHEET_ORACLE_*` environment variables / GitHub `oracle` environment
  secrets (`docs/architecture/oracle.md`). Unblocks T-0004 and its dependents.
- Set `main` as the GitHub default branch (repo Settings → General → Default branch).

## Log
- 2026-10-01 — `main` created; scheduled routines: worker-functions (every 3 h), worker-critical (every 3 h, offset), steward (daily).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
