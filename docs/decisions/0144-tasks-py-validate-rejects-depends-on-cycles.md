# 0144 — tasks.py validate rejects depends_on cycles

- **Status:** accepted (2026-10-28, T-0091). Extends the task schema checks of `scripts/tasks.py validate`
  (decisions 0016, 0017).
- **Context:** `ready()` needs every dependency `done`, so tasks whose `depends_on` lists form a cycle can never be
  picked. On main 899b49fd T-0963 (opus) and T-0965 (sonnet) depended on each other, yet `validate` reported 0 errors:
  it only checked that each dependency id exists. Found by the gap triager on 2026-10-28.
- **Decision:** `validate` (and so `make hygiene`, `make check` and CI) reports every `depends_on` cycle as one error,
  `T-a -> T-b -> T-a: depends_on cycle`, starting at the cycle's smallest id, whatever the tasks' status (a cycle
  through done or dropped tasks is still malformed); a self-dependency is a cycle of one. The T-0963/T-0965 cycle is
  broken by dropping T-0963 from T-0965's `depends_on`: T-0223 landed T-0963's code, and T-0963 only confirms its
  cells once T-0965 adds the workbook.
- **Consequences:** a task-only PR that closes a cycle now fails hygiene and must fix the order before merging. The
  check never weakens a gate; it adds one.
