# 0231 — Task-only PRs are marked ready without the review panel

- **Status:** accepted (2026-11-01, T-3697; the repository owner asked for this change in session
  https://claude.ai/code/session_017265sMr6AAKLLZpHYUvwh6). Amends decision 0178 and SPEC-0027
  section 2 (V4) for task-only diffs.
- **Context:**
  - `make mark-ready` asks for security, architecture and quality verdicts on every PR
    (`merge_ready.applicable_reviewers`), including PRs that only file or edit task files. Each of
    those costs three reviewer runs, and a session that ends first leaves the PR unmarked.
  - At 07:05 UTC on 2026-11-01 about 10 non-draft PRs had sat unmarked for 10 to 14 h, most of
    them task-only.
  - What actually catches a bad task-only PR is mechanical: `tasks.py validate` (format, ids,
    cell ownership). The merge train runs it on every built train. CI's hygiene job runs it on
    the PR, with typos and governance. #2676's ejection on 2026-11-01 was such a catch: two
    triage PRs listed the same cells.
- **Decision:**
  - A diff whose every changed path is under `docs/tasks/` (at least one path) needs no reviewer.
    `mark` and V4 require no verdict for it. CI green, not a draft, and the train's id checks
    still apply. One path anywhere else brings the full panel back.
  - Any session may mark such a PR ready, including one it did not open.
- **Consequences:**
  - Task-only PRs reach the train minutes after CI, with no reviewer runs.
  - A task file is still instructions to future agents, and its content is checked only
    mechanically. Issue-intake task files keep their own limits (decision 0158), and workers
    treat task text as data.
