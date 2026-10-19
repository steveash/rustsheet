# RustSheet developer & agent entry points. `make help` lists targets.
# CI runs `make ci`. Agents MUST run `make check` before opening a PR.

CARGO ?= cargo
UV ?= uv
EVAL := target/release/rustsheet-eval
# Long eval runs go through the crash-evidence wrapper (decision 0041, T-1118).
EVAL_GUARDED := scripts/eval_guard.sh $(EVAL)
PY := python3

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# ---------------------------------------------------------------- Rust
.PHONY: fmt fmt-check clippy test build
fmt: ## Format Rust and Python code
	$(CARGO) fmt --all
	cd tools && $(UV) run ruff format src tests && $(UV) run ruff check --fix src tests

fmt-check: ## Check formatting (CI)
	$(CARGO) fmt --all -- --check

clippy: ## Lint Rust with -D warnings (workspace lint policy in Cargo.toml)
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test: ## Run Rust tests
	$(CARGO) test --workspace --exclude rustsheet_python

build: ## Release build of the eval harness
	$(CARGO) build --release -p rustsheet_eval

# ---------------------------------------------------------------- Eval (Excel compatibility)
.PHONY: eval eval-check eval-baseline eval-show evidence-floor
eval: build ## Run all suites; writes eval/runs/latest/report.{md,json}
	$(EVAL_GUARDED) run

eval-check: build ## Gate: fail on any regression vs eval/baseline (CI)
	$(EVAL_GUARDED) check

eval-baseline: build ## Ratchet eval/baseline and quality/evidence-floor.json after fixes (refuses regressions)
	$(EVAL_GUARDED) update-baseline
	$(EVAL) evidence-floor --update

evidence-floor: build ## Gate: no more in-scope functions below the evidence floor (T-0056, CI)
	$(EVAL) evidence-floor

eval-show: build ## Show one case: make eval-show CASE=ironcalc-calc/FINANCIAL/IRR.xlsx
	$(EVAL) show $(CASE)

# ---------------------------------------------------------------- Python tooling
.PHONY: py-sync py-check py-bindings parity-report
py-sync:
	cd tools && $(UV) sync -q

py-check: py-sync ## Lint, type-check and test tools/
	cd tools && $(UV) run ruff check src tests && $(UV) run ruff format --check src tests && $(UV) run mypy && $(UV) run pytest -q

parity-report: py-sync ## Regenerate docs/parity.md and docs/parity/ from eval/runs/latest/report.json (after make eval) [COMMIT=sha] [RUN_DATE=YYYY-MM-DD]
	cd tools && $(UV) run rustsheet-parity-report --out-dir ../docs $(if $(COMMIT),--commit $(COMMIT)) $(if $(RUN_DATE),--run-date $(RUN_DATE))

py-bindings: ## Build the PyO3 bindings into bindings/python/.venv and run their tests
	cd bindings/python && $(UV) venv -q --allow-existing .venv && $(UV) pip install -q --python .venv maturin pytest
	cd bindings/python && . .venv/bin/activate && maturin develop -q && pytest -q tests

# ---------------------------------------------------------------- Hygiene ratchets
.PHONY: hygiene deny machete typos
hygiene: ## Lint-debt, file-size, duplication, tasks and governance ratchets
	$(PY) scripts/check_lint_debt.py
	$(PY) scripts/check_file_size.py
	$(PY) scripts/check_duplication.py
	$(PY) scripts/tasks.py validate
	$(PY) scripts/check_decisions.py
	$(PY) scripts/check_docs_readers.py
	$(PY) -m unittest discover -s scripts -p 'test_*.py' -q

deny: ## License/advisory/source policy (needs cargo-deny)
	$(CARGO) deny check

machete: ## Unused dependencies (needs cargo-machete)
	cargo machete

typos: ## Spelling (needs typos-cli)
	typos

governance: ## Protected-path changes need a decision record
	$(PY) scripts/check_governance.py $(BASE)

# ---------------------------------------------------------------- Aggregates
.PHONY: check ci
check: fmt-check clippy test eval-check evidence-floor py-check py-bindings hygiene typos ## Everything an agent must pass before a PR (typos too: CI fails on it)

ci: check deny machete

# ---------------------------------------------------------------- Tasks
.PHONY: tasks next-task ready-tasks board new-task new-decision
tasks: ## List tasks (LANE=, STATUS=, MILESTONE= filters)
	$(PY) scripts/tasks.py list $(if $(LANE),--lane $(LANE)) $(if $(STATUS),--status $(STATUS)) $(if $(MILESTONE),--milestone $(MILESTONE))

next-task: ## Best ready task in a non-busy lane: make next-task LANE=functions [TIER=sonnet|opus] (LANE=functions = any functions-* lane)
	$(PY) scripts/tasks.py next $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

ready-tasks: ## Every task next-task could pick, then "N ready": make ready-tasks TIER=sonnet [LANE=..] (decision 0021)
	$(PY) scripts/tasks.py ready $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

new-task: ## New task file with a collision-free id: make new-task MILESTONE=M1 LANE=engine TITLE="..." [PRIORITY=P1] [SIZE=S] [TIER=opus] (fetches origin)
	$(PY) scripts/tasks.py new --milestone "$(MILESTONE)" --lane "$(LANE)" --title "$(TITLE)" $(if $(PRIORITY),--priority $(PRIORITY)) $(if $(SIZE),--size $(SIZE)) $(if $(TIER),--tier $(TIER))

new-decision: ## New decision record with a collision-free number: make new-decision TITLE="..." (fetches origin; decision 0029)
	$(PY) scripts/new_decision.py --title "$(TITLE)"

board: ## Write docs/tasks/BOARD.md locally (git-ignored; decision 0017)
	$(PY) scripts/tasks.py board

# ---------------------------------------------------------------- Merge lock (decision 0075)
.PHONY: merge-lock-acquire merge-lock-release merge-lock-status
merge-lock-acquire: ## Take (or renew) the merge lock before merging a PR: make merge-lock-acquire PR=123 (exit 1: busy)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_lock.py acquire --pr '$(PR)'

merge-lock-release: ## Free your merge lock (after the merge, a red CI, or any stop)
	$(PY) scripts/merge_lock.py release

merge-lock-status: ## Who holds the merge lock, and since when
	$(PY) scripts/merge_lock.py status
