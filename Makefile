# RustSheet developer & agent entry points. `make help` lists targets.
# CI runs `make ci`. Agents MUST run `make check` before opening a PR.

CARGO ?= cargo
UV ?= uv
EVAL := target/release/rustsheet-eval
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
.PHONY: eval eval-check eval-baseline eval-show
eval: build ## Run all suites; writes eval/runs/latest/report.{md,json}
	$(EVAL) run

eval-check: build ## Gate: fail on any regression vs eval/baseline (CI)
	$(EVAL) check

eval-baseline: build ## Ratchet eval/baseline after fixes (refuses regressions)
	$(EVAL) update-baseline

eval-show: build ## Show one case: make eval-show CASE=ironcalc-calc/FINANCIAL/IRR.xlsx
	$(EVAL) show $(CASE)

# ---------------------------------------------------------------- Python tooling
.PHONY: py-sync py-check py-bindings
py-sync:
	cd tools && $(UV) sync -q

py-check: py-sync ## Lint, type-check and test tools/
	cd tools && $(UV) run ruff check src tests && $(UV) run ruff format --check src tests && $(UV) run mypy && $(UV) run pytest -q

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
check: fmt-check clippy test eval-check py-check py-bindings hygiene ## Everything an agent must pass before a PR

ci: check deny machete typos

# ---------------------------------------------------------------- Tasks
.PHONY: tasks next-task board
tasks: ## List tasks (LANE=, STATUS=, MILESTONE= filters)
	$(PY) scripts/tasks.py list $(if $(LANE),--lane $(LANE)) $(if $(STATUS),--status $(STATUS)) $(if $(MILESTONE),--milestone $(MILESTONE))

next-task: ## Highest-priority ready task: make next-task LANE=functions
	$(PY) scripts/tasks.py next $(if $(LANE),--lane $(LANE))

board: ## Regenerate docs/tasks/BOARD.md
	$(PY) scripts/tasks.py board
