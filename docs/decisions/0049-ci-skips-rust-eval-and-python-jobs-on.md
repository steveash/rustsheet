# 0049 — CI skips Rust, eval and Python jobs on docs-only PRs

- **Status:** accepted (2026-10-12, T-1415; owner request: "cut CI cost by avoiding rust stuff for
  doc only changes")
- **Context:** About a third of agent PRs touch only task files, status, decisions or skills, yet
  every PR ran all four CI jobs (rust fmt/clippy/test, eval gate, python tools and bindings,
  hygiene), about 4 minutes each. The repo ran ~1,700 CI runs in its first 9 days and exhausted the
  private-repo Actions allowance on 2026-10-09; it is public now, but CI time still serializes the
  merge queue (branches must be up to date with `main`, so merges happen one CI run at a time).
- **Decision:** A new `changes` job classifies the PR's changed files. When every file is under
  `docs/` (except `docs/excel-semantics/`), under `.claude/`, or a top-level `*.md`, the `rust`,
  `eval` and `python` jobs are skipped; `hygiene` (task and decision validation, governance, typos,
  ratchets) always runs. Pushes to `main`, `workflow_dispatch`, an empty or unreadable diff, and a
  failed `changes` job all run everything.
- **Consequences:**
  - A skipped job reports success to required status checks, so job names and branch protection
    are unchanged. The fail-safe is the `if:` condition: the jobs run unless `changes` succeeded
    and said `code=false`.
  - `docs/excel-semantics/` always runs the full CI because the eval harness validates DEVIATION
    anchors in it (`crates/rustsheet-eval/src/deviations.rs`). Any future code or test that reads a
    file under `docs/` or `.claude/` at build or test time must add that path to the exception
    list in the same PR (since decision 0066: `CODE_READ_PATHS` in `scripts/ci_changes.py`,
    enforced by `scripts/check_docs_readers.py`).
  - The diff uses `--no-renames`, so a rename lists both the old and the new path: moving a code
    file into `docs/` still counts as code (security and architecture review).
  - `make check` locally is unchanged.
