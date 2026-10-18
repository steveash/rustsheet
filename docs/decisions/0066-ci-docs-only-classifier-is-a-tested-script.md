# 0066 — CI docs-only classifier is a tested script run from the base commit

- **Status:** accepted (2026-10-16, T-1680; amends 0049)
- **Context:** Decision 0049's classifier was an inline `case` statement in the `changes` job of
  `.github/workflows/ci.yml`. It had no test, and a wrong skip looks exactly like a pass. Its
  safety also rested on a convention: a new build- or test-time reader of `docs/` or `.claude/`
  had to add itself to the exception list, and nothing checked that. The long `if:` condition was
  copied into the three heavy jobs. The 2026-10-16 steward spot-audit of PR #606 found all three
  (security, architecture and quality reviewers, MINOR each).
- **Decision:**
  - The rules live in `scripts/ci_changes.py` (NUL-separated paths on stdin, `code=true|false` on
    stdout), tested by `scripts/test_ci_changes.py` under `make hygiene`. Which paths count as
    docs-only is unchanged.
  - CI runs the copy from the PR's base commit (`git show $BASE:scripts/ci_changes.py`), so a PR
    cannot change how its own files are classified; a base without the script runs everything.
  - `scripts/check_docs_readers.py` (in `make hygiene`) fails when code under `crates/`, `tools/`,
    `bindings/` or `fuzz/` names a repository `docs/` or `.claude/` path in a string literal that
    is not under `CODE_READ_PATHS` in `scripts/ci_changes.py`. That list is the one exception
    list: the classifier and the check share it.
  - The `changes` job exposes one output, `run_heavy`, and the heavy jobs share one condition:
    `!cancelled() && needs.changes.outputs.run_heavy != 'false'`. The output is `'true'` unless the
    classification step itself succeeded with `code=false`, and empty when the job never ran.
- **Consequences:**
  - Every fail-safe of 0049 stays: pushes, merge groups, `workflow_dispatch`, a missing base, a
    failed `git show` or `git diff`, a failed script and a failed or skipped `changes` job all run
    everything.
  - A change to the classification rules takes effect for PRs opened against the commit that
    merged it, not for the PR that makes it.
  - The literal scan is a heuristic: it misses a path assembled from fragments other than `"docs"`
    (which it flags). A literal that looks like a repository docs path but is not (a crate's own
    `docs/` directory) is listed in `NOT_REPO_DOCS` with the reason.
