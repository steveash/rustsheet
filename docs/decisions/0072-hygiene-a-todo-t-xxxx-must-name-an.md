# 0072 — Hygiene: a TODO(T-XXXX) must name an open task

- **Status:** accepted (2026-10-18, T-1779)
- **Context:** CLAUDE.md allows only `TODO(T-0123)` so every TODO has an owner, but nothing checked
  that the owner is still open. On 2026-10-17 six TODOs named done tasks (resolved by T-1783); by the
  time this check landed, four more had appeared (T-1858 and T-1865 were done while code still named
  them). A TODO whose task is closed has no owner, and nobody picks the work up.
- **Decision:** `scripts/tasks.py validate` (so `make hygiene`, `make check` and CI) fails on every
  `TODO(T-XXXX)` in a tracked file under crates/, bindings/, tools/ or scripts/ whose task is done,
  dropped or missing, naming file:line and the status. docs/ is left out: specs and task files quote
  old TODOs as history. This touches protected paths (scripts/) and adds a check; it weakens nothing.
- **Consequences:** a PR that closes a task must resolve or re-point the TODOs that name it, or its
  `make check` fails. A PR that writes a TODO must name an existing open task (create it with
  `make new-task` first). T-1779 re-pointed the four stale TODOs it found: fisher.rs and its test to
  T-1864 (F.DIST's pdf at large degrees of freedom), locale_tag.rs to T-1875 (regional LCIDs).
