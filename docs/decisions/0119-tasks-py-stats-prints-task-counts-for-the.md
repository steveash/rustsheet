# 0119 — tasks.py stats prints task counts for the progress dashboard

- **Status:** accepted (2026-10-25, T-0069)
- **Context:** The progress dashboard (SPEC-0022, "Task counts") shows task counts per milestone
  and per campaign stage. Its generator must not parse task front matter itself, because
  scripts/tasks.py is the one definition of a task, of `ready` and of the campaign stages
  (decisions 0083, 0114). scripts/ is a protected path.
- **Decision:** `scripts/tasks.py stats --json` prints one JSON object with sorted keys:
  `by_milestone` (milestone -> status -> count), `by_campaign` (stage -> status -> count, plus
  `ready`) and `deferred` (the same status map plus `ready`, for deferred tasks). It reuses `load()`, `ready()`,
  `CAMPAIGNS` and `OPEN`. Every status and every stage is listed, even at zero, so the shape does
  not change from day to day.
  - A task counts toward a stage by its own `campaign:` tag. The open dependencies that `next`
    also orders as that stage (`in_campaign`, decision 0094) are not counted, because those
    have no stage once they are done.
  - A task with both `campaign:` and `defer:` counts in its stage and in `deferred`, and `ready`
    excludes it from both. Consumers must not sum the stage rows and the parked row.
  - `deferred` keeps every status, so the parked row can show open (todo, in-progress and
    blocked), blocked and done like a stage; its `ready` is always 0.
  - `ready` uses `ready()` with no branch-claim check, so it never fetches. It is the tasks that
    are ready on main, not the tasks a worker may start. The dashboard says so (SPEC-0022).
  - It runs the same checks as `validate`, and on any error it prints them to stderr, prints no
    counts and exits 1. Counts over an invalid task set would mislead the dashboard, and the
    counts index the status maps directly, so an unknown status must be rejected first.
  - No fetch and no network. Its one git call is the stale-TODO scan's local, read-only
    `git ls-files`; if git is missing or fails, it reports that and exits 1.
- **Consequences:**
  - The dashboard generator (T-0072) reads this output and never reads task files.
  - A new status, campaign stage or deferral shows up in the output with no change to the
    generator's parsing.
  - The command needs `--json`; without it, it prints usage and exits 2. That leaves room for a
    human-readable form later.
