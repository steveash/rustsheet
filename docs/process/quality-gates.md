# Quality gates

Every gate is automated and runs in CI (`.github/workflows/ci.yml`). `make check` runs the same
gates locally, except `cargo deny` and `cargo machete` (`make ci`). Gates are **ratchets**: a number that may only improve. Lowering a bar requires a
decision record in `docs/decisions/` (enforced by `scripts/check_governance.py`).

| Gate | Command | Rule |
|---|---|---|
| Formatting | `make fmt-check`, `ruff format --check` | rustfmt / ruff formatted |
| Rust lints | `make clippy` | workspace policy in `Cargo.toml` (`clippy::pedantic`, `unwrap_used`, `panic`, `print_*`, `unsafe_code = deny`), `-D warnings` |
| Lint debt | `scripts/check_lint_debt.py` | per-crate `LINT-DEBT` allow-lists (inherited from the fork) may only shrink; locked in `quality/lint-debt.json` |
| Rust tests | `make test` | all pass |
| **Excel compatibility** | `make eval-check` | no regressions against `eval/baseline/*.txt` (known failures); baseline must be up to date |
| Python tools | `make py-check` | ruff, mypy `--strict`, pytest |
| Python bindings | `make py-bindings` | maturin build + pytest |
| Duplication | `scripts/check_duplication.py` | duplicated lines (10-line normalized windows) may not increase; `quality/duplication.json` |
| File size | `scripts/check_file_size.py` | new source files ≤ 1000 lines; inherited oversized files may not grow; `quality/oversized-files.json` |
| Tasks | `scripts/tasks.py validate` | task schema, ids, dependencies, no unsplit L tasks |
| Dependencies | `cargo deny check` | license allow-list, RustSec advisories, sources, no wildcard versions |
| Unused deps | `cargo machete` | none |
| Spelling | `typos` | clean (Excel names allow-listed in `_typos.toml`) |
| Governance | `scripts/check_governance.py origin/main` | protected-path changes need a decision record; test deletions flagged |

## Ratchet mechanics

- **Eval baseline** (`eval/baseline/<suite>.txt`): records each case's checked-cell count and the
  exact known-failing cells.
  - `make eval-check` fails if a cell that is not listed now fails in an unchanged case
    (regression), or if the file is out of date because cells were fixed or new cases were added.
  - `make eval-baseline` rewrites it and refuses regressions unless `--allow-regressions` is
    passed. That flag is only legitimate with a decision record, for example when an oracle
    re-run shows the old expectation was wrong.
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
