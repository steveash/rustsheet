---
id: T-3701
title: "CI skips the Rust test shards for PRs that change no Rust input"
milestone: M0
lane: harness
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A PR whose diff cannot change what the Rust tests, clippy or the Python bindings see (for example
only eval suite data, eval baselines or `docs/excel-semantics/`) skips the Rust test shards and
the bindings build, and still runs the eval gate and hygiene. Anything the classifier cannot
place still runs everything.

## Context
Owner request (2026-11-01, CI-speed investigation). The `changes` job (decision 0049, rules in
`scripts/ci_changes.py`, decision 0066) gives one answer, docs-only or not. Of the last 52 commits
on main, 38 ran the heavy jobs, but only 26 touched `crates/` or `bindings/`; 7 touched only eval
data or `docs/excel-semantics/` and paid for about 9.5 min of Rust tests anyway. Some Rust tests
and build scripts read files outside `crates/` (for example
`crates/rustsheet-xlsx/build.rs` reads `eval/suites/ironcalc-calc/cached-overrides.tsv`; the eval
crate's tests may read `eval/` and `docs/excel-semantics/` anchors), so the classifier needs an
exact, tested list of such inputs, like `CODE_READ_PATHS` and `scripts/check_docs_readers.py`.

## Acceptance criteria
- [ ] `ci_changes.py` emits separate outputs (at least `rust` and `eval`/`python`) and CI keys each
      job on its own output; the fail-safe of 0049 holds per output (unknown, empty or failed
      classification runs everything; pushes to main and the train branches run everything).
- [ ] Every file a Rust test, build script or the bindings read outside `crates/` and `bindings/`
      is in a tested list, and a hygiene check fails when a new reader is added without it.
- [ ] Unit tests in `scripts/test_ci_changes.py` cover each new class and the fail-safes.
- [ ] Required checks still report (skipped jobs pass via the summary job, as in 0122).
- [ ] A decision record amends 0049/0066; `make check` green; reviewers approve.

## Out of scope
- Changing which tests run inside a job.

## Blocked because
Blocked because: needs an owner ruling. The measured gain is smaller than this task assumed, and
its main safeguard cannot be made reliable.

Evidence (2026-11-01). Every test binary of `make test` except the engine's unit tests ran under
`strace -f`, and the sources were grepped. Rust reads these paths outside crates/ and bindings/:
- `eval/suites/fixtures/`
- `eval/suites/probes/cases/` (test_move_formula.rs, test_legacy_n_t_marker.rs)
- `eval/suites/ironcalc-calc/cached-overrides.tsv` (crates/rustsheet-xlsx/build.rs)
- `eval/catalog/excel-functions.tsv` (functions/kinds/arg_counts.rs)
- the rustsheet-eval unit tests, through `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")`:
  `eval/baseline/visible-parity.txt`, `eval/catalog/scope.json`, `eval/catalog/lift-kinds.json`,
  and `eval/milestones/{exit-exceptions,phase1-exceptions,probe-milestones}.tsv`.

Why this blocks the task:
- **The gain is small.** `visible-parity.txt` alone was touched by 6 of the 38 heavy commits
  sampled from main. With every reader above counted as a Rust input, only a few PRs a day would
  skip the shards, and their wall time would drop only from about 6.5 to about 4.5 min, because
  the eval and python jobs still run.
- **The guard is not reliable.** The acceptance criterion "a hygiene check fails when a new
  reader is added" cannot be met statically. A literal scan sees `"../../eval/x"` and
  `concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/x")`, but not a later `root.join(PATH)`, and
  the eval crate uses that form throughout.
- **A wrong skip can reach main.** It looks exactly like a pass (decision 0049), and in `train`
  mode the classifier also decides the train's own run (decision 0147).

What would unblock it, for the owner to choose:
- (a) drop the task;
- (b) skip the shards anyway, guarded only by the literal scan and a periodic strace audit,
  accepting the residual risk;
- (c) first make every Rust test reach repository files through one helper, which a hygiene
  check can then audit.

The branch `agent/harness/T-3701-rust-inputs-ci` holds the unmerged work in progress: the
`rust=` output and its tests in `scripts/ci_changes.py`, and the extended
`scripts/check_docs_readers.py`. Its `RUST_READ_PATHS` still lacks the rustsheet-eval readers
above.
