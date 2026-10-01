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
- Excel oracle access: a Microsoft 365 **work/school** tenant for Graph (see
  `docs/architecture/oracle.md`), or a Windows VM with Excel.
- Confirm creation of the `main` branch and the scheduled worker routines.

## Log
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
