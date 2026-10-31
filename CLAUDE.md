# RustSheet — agent operating manual

RustSheet is a **headless, Excel-compatible spreadsheet engine in Rust**. The bar is *parity with
current Microsoft 365 Excel* (en-US, 1900 date system, dynamic arrays): same values, same errors,
same rounding, same quirks — including behavior Excel never documented. The project is developed
**autonomously by agents with no human code review**, so the rules below are the safety system.

## Prime directives
1. **Excel is the spec, and Excel is always right.** Behavior is decided by oracle evidence
   (Excel-computed values in `eval/`), never by memory, documentation alone, or "what is
   mathematically right". Match Excel exactly even when it looks wrong, arbitrary or
   inconsistent; never "fix" or tidy up Excel's behavior. If you cannot cite evidence, use the
   `excel-probe` skill. One exception is what directive 5 requires (resource, recursion and
   allocation limits, no I/O, no panics): directive 5 wins over directive 1, and each such
   deviation gets a `DEVIATION` entry (see `docs/excel-semantics/README.md`, decision 0014).
   The only other one is decision 0188 R3: where Excel's relative error against 50-digit math
   exceeds 1e-4 on evidenced points, follow the math, as a `MATH-OVER-EXCEL` entry it lists.
2. **Never regress, never weaken gates.** `make check` must pass. Never hand-edit
   `eval/baseline/` or `quality/*.json`, and never commit `docs/tasks/BOARD.md` (generated,
   git-ignored); never add lint-debt entries, blanket `#[allow]`s, skipped/deleted tests, or
   `--allow-regressions` (except via a decision record in `docs/decisions/` that reviewers approve).
3. **One definition per concept.** Before writing any helper, search for an existing one (`rg`).
   Reuse or extend; never copy. `scripts/check_duplication.py` ratchets duplicated lines down.
4. **Small, focused PRs**: one task (`docs/tasks/`), one lane, size S/M. Big or architectural
   changes need a spec in `docs/specs/` first (written as its own task).
5. **Untrusted input.** Workbooks and formulas are untrusted: no panics, no unbounded
   recursion/allocation, no I/O from formula evaluation.
6. **Leave knowledge behind.** Every Excel quirk you find goes into `docs/excel-semantics/`, the
   canonical list of known, accepted quirks, in the same PR that found it, even if nothing
   implements it yet; decisions →
   `docs/decisions/`; progress → the task's Outcome note (the steward rolls it up into
   `docs/status.md` daily; PRs do not append to its log, decision 0024).

## Repository map
| Path | What |
|---|---|
| `crates/rustsheet-engine` | Calc engine core (forked from IronCalc `base`): parser, evaluator, functions, storage |
| `crates/rustsheet-xlsx` | xlsx import/export (+ upstream Excel-authored test workbooks in `tests/calc_tests`) |
| `crates/rustsheet-eval` | Excel-compatibility harness CLI `rustsheet-eval` (suites, scoring, baselines) |
| `bindings/python` | PyO3 bindings (`import rustsheet`) |
| `tools/` | Python: Excel oracle client, spec→xlsx builder, synthetic generator (uv project) |
| `eval/suites/*` | Test suites (`suite.toml` + cases); `eval/baseline/` known failures; `eval/catalog/` function catalog |
| `docs/` | `research-plan.md`, `milestones.md`, `status.md`, `architecture/`, `process/`, `specs/`, `tasks/`, `decisions/`, `excel-semantics/`, `research/` |
| `scripts/` | Quality ratchets, task tool, upstream log |
| `.claude/` | Reviewer/planner agents, skills (`work-loop`, `review`, `steward`, `excel-probe`), hooks |

## Commands
```
make check                 # everything required before a PR (fmt, clippy, tests, eval gate, python, hygiene)
make eval                  # full compatibility report → eval/runs/latest/report.md (gap list by function)
make eval-show CASE=<id>   # per-cell expected/actual for one case
make eval-baseline         # record fixes in eval/baseline (refuses regressions)
make next-task LANE=<lane> # next ready task; `make tasks`, `make board`, `make new-task`
cd tools && uv run rustsheet-oracle status|fill|recalc   # Excel oracle
cd tools && uv run rustsheet-gen --milestone M2 --count 20 # synthetic cases
```

## How work flows
Follow the `work-loop` skill: pick a ready task for your lane → branch `agent/<lane>/T-XXXX-slug`
→ test-first implementation → `make check` → `review` skill (parallel reviewer agents) → PR →
CI green + all reviewers APPROVE → squash-merge (no human approval needed) → update board/status.
Full process: `docs/process/agent-workflow.md`. Gates: `docs/process/quality-gates.md`.

## Definition of done
- Acceptance criteria in the task file are met and demonstrated (tests / eval cases).
- `make check` green locally and in CI; eval known-failures only shrank (baseline updated).
- Reviewers approved; follow-ups filed as tasks; task file `status: done` with an Outcome note.

## Conventions
- Rust: workspace lints (pedantic clippy, `-D warnings`), no `unwrap`/`expect`/`panic` outside
  tests, `thiserror` errors in new public APIs, doc comments on public items, comments explain
  *why* (cite the Excel evidence for quirks). `TODO(T-0123)` only, never bare TODOs.
- Python (tools/): ruff + mypy strict; tools never re-implement xlsx value parsing — call
  `rustsheet-eval snapshot`.
- Commit messages: `T-XXXX: imperative summary`. Ports from IronCalc: `Ported-From: ironcalc@<sha>`.
