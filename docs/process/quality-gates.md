# Quality gates

Every gate is automated and runs in CI (`.github/workflows/ci.yml`). `make check` runs the same
gates locally, except `cargo deny` and `cargo machete` (`make ci`). Gates are **ratchets**: a number that may only improve. Lowering a bar requires a
decision record in `docs/decisions/` (enforced by `scripts/check_governance.py`).
On docs-only PRs (decision 0049) CI skips the rust, eval and python jobs; hygiene always runs, and
`docs/excel-semantics/` counts as code because the eval harness reads it. The rules are
`scripts/ci_changes.py`, run from the PR's base commit (decision 0066).

| Gate | Command | Rule |
|---|---|---|
| Formatting | `make fmt-check`, `ruff format --check` | rustfmt / ruff formatted |
| Rust lints | `make clippy` | workspace policy in `Cargo.toml` (`clippy::pedantic`, `unwrap_used`, `panic`, `print_*`, `unsafe_code = deny`), `-D warnings` |
| Lint debt | `scripts/check_lint_debt.py` | per-crate `LINT-DEBT` allow-lists (inherited from the fork) may only shrink; locked in `quality/lint-debt.json` |
| Rust tests | `make test` | all pass |
| **Excel compatibility** | `make eval-check` | no regressions against `eval/baseline/*.txt` (known failures); baseline must be up to date |
| Evidence floor | `make evidence-floor` | in-scope, oracle-evaluable functions with fewer than 20 Excel-evidence cells may not increase; `quality/evidence-floor.json` (decision 0062) |
| Python tools | `make py-check` | ruff, mypy `--strict`, pytest |
| Python bindings | `make py-bindings` | maturin build + pytest |
| Duplication | `scripts/check_duplication.py` | duplicated lines (10-line normalized windows) may not increase; `quality/duplication.json` |
| File size | `scripts/check_file_size.py` | new source files ≤ 1000 lines; inherited oversized files may not grow; `quality/oversized-files.json` |
| Tasks | `scripts/tasks.py validate` | task schema, ids, dependencies, no unsplit L tasks |
| Dependencies | `cargo deny check` | license allow-list, RustSec advisories, sources, no wildcard versions |
| Unused deps | `cargo machete` | none |
| Spelling | `typos` | clean (Excel names allow-listed in `_typos.toml`) |
| Docs readers | `scripts/check_docs_readers.py` | code may read `docs/` or `.claude/` only under `CODE_READ_PATHS` in `scripts/ci_changes.py`, which CI never skips (decision 0066) |
| Governance | `scripts/check_governance.py origin/main` | protected-path changes need a decision record; test deletions flagged |

## Ratchet mechanics

- **Eval baseline** (`eval/baseline/<suite>.txt`): records each case's checked-cell count and the
  exact known-failing cells.
  - `make eval-check` fails if a cell that is not listed now fails in an unchanged case
    (regression), or if the file is out of date because cells were fixed or new cases were added.
  - `make eval-baseline` rewrites it and refuses regressions unless `--allow-regressions` is
    passed. That flag is only legitimate with a decision record, for example when an oracle
    re-run shows the old expectation was wrong.
  - Displayed text (a sidecar's `text` map, decision 0068) is gated the same way on its own lines,
    `TC` (a case's scored text cells) and `TF` (a known text failure), in every gated suite
    (decision 0070).
- **Accepted deviations** (`eval/suites/<suite>/deviations`, decision 0036): cells that differ
  from Excel by design (a `Status: DEVIATION` entry, decision 0014), each with RustSheet's pinned
  by-design value.
  - A listed cell giving the pinned value scores as a *deviation*: the report's `deviations`
    column, outside the bit-exact denominator, never an F line. Any other wrong value is a
    failure. A listed cell matching Excel passes with a stale-line warning; remove that line.
  - `make eval-check` and `make eval` fail when a line is malformed, names a missing case or an
    unscored cell, or cites an entry without a `Status: DEVIATION` line.
  - Changing the file is a protected change (`scripts/check_governance.py` asks for a decision
    record): the DEVIATION entry lands in the same PR, and the
    Excel-fidelity reviewer confirms the limit is a directive-5 limit.
- **Evidence floor** (`quality/evidence-floor.json`, decision 0062): `rustsheet-eval
  evidence-floor` runs the vendored suites (fetched external suites do not count) and counts the
  in-scope catalog functions the oracle can evaluate with fewer than 20 Excel-evidence cells. It
  fails when the count rises and when it falls without the lock being updated; `make
  eval-baseline` records a drop. Recording a rise needs `--update --allow-increase` and a
  decision record (for example, when the catalog brings functions into scope).
- **Lint debt / duplication / oversized files:** the scripts fail on increases. When you reduce
  them, run the script with `--update` and commit the JSON in the same PR.

## What reviewers check that tools cannot
See `.claude/agents/reviewer-*.md`:
- semantic duplication
- layering
- spec adherence
- Excel evidence
- panic-on-untrusted-input paths
- complexity and allocation
- test quality

## Planned additions

| Milestone | Gate |
|---|---|
| M1 | public-API docs coverage (`missing_docs` on the facade crate) |
| M3 | benchmark regression gate (> 5% slower fails), metamorphic incremental-vs-full suite |
| M4 | memory-per-cell budget test |
| M5 | coverage report (cargo-llvm-cov) with a ratchet on engine line coverage |
| M12 | `cargo semver-checks`, 24 h fuzzing (parser, xlsx importer) |
