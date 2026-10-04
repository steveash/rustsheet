# 0017 — The task board is generated locally, not committed

- **Status:** accepted (2026-10-04; T-0016)
- **Context:**
  - `docs/tasks/BOARD.md` was generated from the task files and committed. Every PR that touches a
    task (nearly all of them: claim, done, new tasks) rewrites its counts and rows, so concurrent PRs
    conflicted on it. On 2026-10-04 #44, #46, #49 and the T-0022 branch each needed a merge, a
    regenerate, a `make check` and a fresh CI run for this alone.
  - A git merge driver does not help: GitHub computes mergeability server-side without it, so the
    PR still shows a conflict and the API merge is refused.
- **Decision:**
  - `docs/tasks/BOARD.md` is git-ignored and removed from the repository. `make board` still writes
    it locally for anyone who wants the view; `make tasks` / `make next-task` read the task files.
  - The task files stay the single source of truth; `tasks.py validate` still runs in CI.
  - Docs, skills and CLAUDE.md no longer ask agents to run `make board` before committing.
- **Consequences:**
  - No more board conflicts; task-file changes in different files merge cleanly.
  - There is no rendered board on GitHub. The owner reads `docs/status.md` (steward snapshot) or
    runs `make board` / `make tasks`.
