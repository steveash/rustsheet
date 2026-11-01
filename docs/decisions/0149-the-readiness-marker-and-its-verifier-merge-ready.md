# 0149 — The readiness marker and its verifier: merge_ready.py verify

- **Status:** accepted (2026-10-28, T-2986). It implements SPEC-0027 section 2 ("The readiness
  handoff", checks V1-V6) and section 4 step 6 (the "green" rule), and is not in force for any
  merge until `scripts/merge_train.toml`'s mode says so (T-2996). Amended by 0161 (`mark`).
- **Context:** The merge train needs a mechanical check that a PR was marked ready by a writer
  for its current head, is green and approved, without reviewing it again. T-3001's probe found
  that the session proxy refuses the permission API (HTTP 403), which V1 and V6 were specified
  to use. The job log where CI prints `run_heavy` is also unreadable from a session: the log
  download is a redirect to blob storage that the proxy does not reach (tried on job
  122851242233, 2026-10-28).
- **Decision:** `scripts/merge_ready.py` holds the single definitions of the
  `merge-train-ready v1` marker, reviewer applicability (a copy of the `review` skill's step 3,
  pinned to it by a test) and the "green" rule; `verify --pr N` runs V1-V6 and only reads.
  - An author counts as a writer when the permission API says `admin`, `maintain` or `write`.
    When that read fails, only `author_association` `OWNER` or `COLLABORATOR` counts, and only on
    a repository whose owner is a user: there every collaborator can push. On an organization's
    repository a failed permission read means outsider (fail closed).
  - `run_heavy=false` (skipped eval and python jobs count as green) is told by running
    `ci_changes.classify_range` on the PR's `base.sha` and head, the classifier CI runs, instead
    of reading the job log. Unknown means `run_heavy=true`, so a skip is red.
  - Each failed check carries its side effect (ignore, keep the label, unlabel with a comment);
    the merger (T-2987) applies them. Only the merger's own constant texts and shas go into its
    reasons, never user text.
  - `mark` (the label and the marker comment, `--carry`, the step-mode write fallback) is split
    out to T-3040, as T-2986 allowed past size M.
- **Consequences:**
  - `.claude/skills/review/SKILL.md` is in `CODE_READ_PATHS` (decision 0066): a PR that changes
    only it runs the heavy jobs, so the pinning test runs.
  - Changing which reviewers apply means changing the skill and `merge_ready.py` together.
  - `scripts/gh_client.py` gains the reviews read.
  - `make merge-ready-verify PR=<n>` exists; no skill or routine uses it yet (T-2989).
