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
.PHONY: fmt fmt-check clippy test test-shard-1 test-shard-2 test-shard-3 build
fmt: ## Format Rust and Python code
	$(CARGO) fmt --all
	cd tools && $(UV) run ruff format src tests && $(UV) run ruff check --fix src tests

fmt-check: ## Check formatting (CI)
	$(CARGO) fmt --all -- --check

clippy: ## Lint Rust with -D warnings (workspace lint policy in Cargo.toml)
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test: ## Run Rust tests
	$(CARGO) test --workspace --exclude rustsheet_python

# CI runs `make test` as three shards (decisions 0122 and 0232): shard 1 runs the tests whose names
# contain one of the TEST_SHARD_1 substrings, shard 2 the tests matching TEST_SHARD_2 but not
# TEST_SHARD_1, shard 3 every other test (`--skip` matches the same way), so together they run
# exactly the tests of `make test`, each once; a new test lands in shard 3 unless it matches.
# Memory-bound whole-grid tests slow each other down on one runner: keep test_fn_groupby in
# shard 1, apart from test_fn_pivotby and every test_tailed_array test, and split
# test_tailed_array between shards 2 and 3. Decision 0122 says when to rebalance.
TEST_SHARD_1 := test::test_fn_groupby text_functions::fn_regex test::test_multi_area_arguments dynamic_evaluation::test_range_reads functions::text:: test::lookup_and_reference test::test_collation test::test_formula_work test::statistical
TEST_SHARD_2 := test::test_fn_pivotby test_tailed_array::keyed test_tailed_array::index test_tailed_array::lookup test_tailed_array::small_and_large test::test_nested_groupby_cells
SHARD_TEST := $(CARGO) test --workspace --exclude rustsheet_python --

test-shard-1: ## CI: the tests matching TEST_SHARD_1 (a third of make test)
	$(SHARD_TEST) $(TEST_SHARD_1)

test-shard-2: ## CI: the tests matching TEST_SHARD_2 and not TEST_SHARD_1 (a third of make test)
	$(SHARD_TEST) $(TEST_SHARD_2) $(addprefix --skip ,$(TEST_SHARD_1))

test-shard-3: ## CI: every test shards 1 and 2 do not run (the last third of make test)
	$(SHARD_TEST) $(addprefix --skip ,$(TEST_SHARD_1) $(TEST_SHARD_2))

build: ## Release build of the eval harness
	$(CARGO) build --release -p rustsheet_eval

# ---------------------------------------------------------------- Eval (Excel compatibility)
.PHONY: eval eval-check eval-gates eval-baseline eval-lower-visible-lock eval-show evidence-floor
eval: build ## Run all suites; writes eval/runs/latest/report.{md,json}
	$(PY) scripts/desktop_owned.py --out eval/runs/desktop-owned.tsv
	$(EVAL_GUARDED) run --desktop-owned

eval-check: build ## Gate: fail on any regression vs eval/baseline
	$(EVAL_GUARDED) check

eval-gates: build ## eval-check and evidence-floor from one run of the suites, then Gate A's accuracy check and new-case owners (CI, make check; T-2761, decisions 0193, 0245)
	$(EVAL_GUARDED) check --evidence-floor
	$(EVAL_GUARDED) accuracy
	@# Cases the branch adds or refills: every visibly-off cell needs an open owner (T-3793, decision 0245).
	$(PY) scripts/new_case_owners.py --eval $(EVAL)

eval-baseline: build ## Ratchet eval/baseline and quality/evidence-floor.json after fixes (refuses regressions); NEW_CASES=<file> lists the only cases new to the visible-parity lock (make catch-up, decision 0217)
	$(EVAL_GUARDED) update-baseline $(if $(NEW_CASES),--new-cases '$(NEW_CASES)')
	$(EVAL) evidence-floor --update

eval-lower-visible-lock: build ## Steward only, daily: also lower the cases of eval/baseline/visible-parity.txt (decisions 0093, 0217)
	$(EVAL_GUARDED) update-baseline --lower-visible-lock

evidence-floor: build ## Gate: no more in-scope functions below the evidence floor (T-0056; CI runs it in eval-gates)
	$(EVAL) evidence-floor

eval-show: build ## Show one case: make eval-show CASE=ironcalc-calc/FINANCIAL/IRR.xlsx
	$(EVAL) show $(CASE)

# ---------------------------------------------------------------- Python tooling
.PHONY: py-sync py-check py-bindings progress-report
py-sync:
	cd tools && $(UV) sync -q

py-check: py-sync ## Lint, type-check and test tools/
	cd tools && $(UV) run ruff check src tests && $(UV) run ruff format --check src tests && $(UV) run mypy && $(UV) run pytest -q

progress-report: py-sync ## Regenerate docs/progress.md and docs/progress/ from eval/runs/latest/ (after make eval; SPEC-0022, decision 0120)
	mkdir -p eval/runs/latest
	$(PY) scripts/tasks.py stats --json > eval/runs/latest/tasks.json.tmp || { rm -f eval/runs/latest/tasks.json.tmp; exit 1; }
	mv eval/runs/latest/tasks.json.tmp eval/runs/latest/tasks.json
	cd tools && $(UV) run rustsheet-progress-report --out-dir ../docs

py-bindings: ## Build the PyO3 bindings into bindings/python/.venv and run their tests
	cd bindings/python && $(UV) venv -q --allow-existing .venv && $(UV) pip install -q --python .venv maturin pytest
	cd bindings/python && . .venv/bin/activate && maturin develop -q && pytest -q tests

# ---------------------------------------------------------------- Hygiene ratchets
.PHONY: hygiene deny machete typos
hygiene: ## Lint-debt, file-size, duplication, fitted-constant, tasks and governance ratchets
	$(PY) scripts/check_lint_debt.py
	$(PY) scripts/check_file_size.py
	$(PY) scripts/check_duplication.py
	$(PY) scripts/check_fitted.py
	$(PY) scripts/tasks.py validate
	$(PY) scripts/check_decisions.py
	$(PY) scripts/check_docs_readers.py
	$(PY) scripts/check_ci_workflow.py
	@# The scripts' test modules run in parallel, one process per module, largest file first (decision 0241); xargs fails if any module fails.
	ls -S scripts/test_*.py | xargs -n1 basename | xargs -P "$$(getconf _NPROCESSORS_ONLN)" -I{} $(PY) -m unittest discover -s scripts -p {} -q

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
check: fmt-check clippy test eval-gates py-check py-bindings hygiene typos ## Everything an agent must pass before a PR (typos too: CI fails on it)

ci: check deny machete

# ---------------------------------------------------------------- Tasks
.PHONY: tasks next-task ready-tasks board new-task new-decision exit-owners unowned-off phase1-exit
tasks: ## List tasks (LANE=, STATUS=, MILESTONE= filters)
	$(PY) scripts/tasks.py list $(if $(LANE),--lane $(LANE)) $(if $(STATUS),--status $(STATUS)) $(if $(MILESTONE),--milestone $(MILESTONE))

next-task: ## Best ready task in a non-busy lane: make next-task LANE=functions [TIER=sonnet|opus] (LANE=functions = any functions-* lane)
	$(PY) scripts/tasks.py next $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

ready-tasks: ## Every task next-task could pick, then "N ready": make ready-tasks TIER=sonnet [LANE=..] (decision 0021)
	$(PY) scripts/tasks.py ready $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

new-task: ## New task file with a collision-free id: make new-task MILESTONE=M1 LANE=engine TITLE="..." [PRIORITY=P1] [SIZE=S] [TIER=opus] (fetches origin; write a literal $ as $$)
	$(PY) scripts/tasks.py new --milestone "$$MILESTONE" --lane "$$LANE" --title "$$TITLE" $(if $(PRIORITY),--priority $(PRIORITY)) $(if $(SIZE),--size $(SIZE)) $(if $(TIER),--tier $(TIER))

new-decision: ## New decision record with a collision-free number: make new-decision TITLE="..." (fetches origin; decision 0029)
	$(PY) scripts/new_decision.py --title "$$TITLE"

exit-owners: ## Failing exit cells of a milestone that no open task of it owns, after make eval: make exit-owners MILESTONE=M5 [STRICT=1] (decision 0096)
	$(if $(MILESTONE),,$(error MILESTONE=M<n> is required))
	$(PY) scripts/exit_owners.py --milestone '$(MILESTONE)' $(if $(STRICT),--strict)

unowned-off: ## Visibly-off cells no open task owns, after make eval: unattributed (decision 0132), or ALL=1 [FUNCTION=NAME] every one by function (T-0086)
	$(PY) scripts/exit_owners.py $(if $(ALL),--all-off $(if $(FUNCTION),--function $(FUNCTION)),--unattributed) $(if $(STRICT),--strict)

phase1-exit: ## One verdict per Phase 1 exit criterion after a full make eval; exit 1 unless every measured one is met (decision 0187) [JSON=1]
	$(PY) scripts/phase1_exit.py $(if $(JSON),--json)

board: ## Write docs/tasks/BOARD.md locally (git-ignored; decision 0017)
	$(PY) scripts/tasks.py board

# ---------------------------------------------------------------- Merge lock (decisions 0075, 0145)
.PHONY: merge-lock-acquire merge-lock-wait merge-lock-release merge-lock-status
merge-lock-acquire: ## Take (or renew) the merge lock before merging a PR: make merge-lock-acquire PR=123 [RENEW=1] (exit 1: busy, you are queued; 3: PR already done; 4: lost; 5: refused, train mode)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_lock.py acquire --pr '$(PR)' $(if $(RENEW),--renew)

merge-lock-wait: ## Wait in the queue until the merge lock is yours (run it in the background): make merge-lock-wait PR=123 [TIMEOUT=45] (exit 0: held; 1: timed out; 2: error; 3: PR already done; 5: refused, train mode)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_lock.py wait --pr '$(PR)' --timeout '$(or $(TIMEOUT),45)'

merge-lock-release: ## Free your merge lock (after the merge, a red CI, or any stop)
	$(PY) scripts/merge_lock.py release

merge-lock-status: ## Who holds the merge lock, since when, and the queue of waiters
	$(PY) scripts/merge_lock.py status

.PHONY: merge-ready-verify
merge-ready-verify: ## Check a PR's readiness marker, V1-V6 of SPEC-0027 (reads only): make merge-ready-verify PR=123 (exit 0: ready; 1: not ready; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_ready.py verify --pr '$(PR)'

.PHONY: mark-ready merge-labels verdict-record
catch-up: ## Clear a merge-train ejection for a conflict with main that lies only in eval/baseline/ (decision 0207): make catch-up PR=123 merges main into the PR, re-records the lists and pushes; then make mark-ready PR=123 CARRY=1 once CI is green (exit 0: pushed; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_catchup.py --pr '$(PR)'

renumber: ## Clear a merge-train ejection for an id collision (decision 0224): make renumber PR=123 gives the PR's colliding task and decision files fresh ids from the id registry, rewrites the old id only under docs/ in what the PR added (refusing when a line it added elsewhere names it), merges main in and pushes; then make mark-ready PR=123 CARRY=1 once CI is green (exit 0: pushed; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_renumber.py --pr '$(PR)'

janitor: ## After a merger pass, in train mode (decision 0224): catch up or renumber idle ejected PRs, re-mark them with CARRY once CI is green, re-mark or unlabel stale markers, and comment orphans once; DRY_RUN=1 lists the actions only; STEP=1 prints each GitHub write as a step-mode action to perform and ack with make merge-train-ack (exit 0: done; 2: an error)
	$(PY) scripts/merge_janitor.py $(if $(DRY_RUN),--dry-run) $(if $(STEP),--step)

mark-ready: ## Mark a green, approved PR ready for the merge train (SPEC-0027 section 2): make mark-ready PR=123, its verdicts from the verdict records on the head [REVIEW="security=APPROVE architecture=APPROVE quality=APPROVE [excel-fidelity=APPROVE]" for a reviewer with no record, while verdict_records = warn] (or CARRY=1 after a merge of main that is clean, only re-records eval-gated baseline lists (decision 0207) or comes from make renumber (decision 0224)) [STEP=1: print the writes as step-mode actions] (exit 0: marked; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_ready.py mark --pr '$(PR)' $(if $(CARRY),--carry) $(if $(REVIEW),--review '$(REVIEW)') $(if $(STEP),--step)

verdict-record: ## Post one reviewer's verdict record for the local HEAD, which must be the PR head (decision 0178): make verdict-record PR=123 REVIEWER=security VERDICT=APPROVE REPORT=<file with the reviewer's report> [STEP=1: print the comment as a step-mode action] (exit 0: posted; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(if $(REVIEWER),,$(error REVIEWER=<security|architecture|quality|excel-fidelity> is required))
	$(if $(VERDICT),,$(error VERDICT=<APPROVE|REQUEST_CHANGES> is required))
	$(if $(REPORT),,$(error REPORT=<file> is required))
	$(PY) scripts/merge_ready.py record --pr '$(PR)' --reviewer '$(REVIEWER)' --verdict '$(VERDICT)' --report '$(REPORT)' $(if $(STEP),--step)

merge-labels: ## Create the merge-train labels that are missing (ready-to-merge, merge-train-ejected, merge-train-stop, merge-hold)
	$(PY) scripts/merge_ready.py labels --create

.PHONY: merge-mode merge-train merge-train-ack merge-train-next merge-flip-check merge-train-stats catch-up renumber janitor
merge-train: ## The merge train (SPEC-0027): DRY_RUN=1 prints the batch and writes nothing; SHADOW=1 tests and predicts on merge-train/shadow, in shadow mode only; TRAIN=1 merges a green train, in train mode only; STEP=1 (with SHADOW or TRAIN) prints each write as a JSON action for the session to perform and ack
	$(if $(DRY_RUN)$(TRAIN)$(SHADOW),,$(error DRY_RUN=1 or SHADOW=1 or TRAIN=1 is required))
	$(PY) scripts/merge_train.py $(if $(DRY_RUN),--dry-run,$(if $(SHADOW),--shadow,--train) $(if $(STEP),--step))

merge-train-ack: ## Step mode (decision 0169): the session performed the pending action: make merge-train-ack SEQ=3 [REFUSED=1: GitHub refused the write]; then waits up to NEXT=90 s for the next action and prints it (decision 0238) (exit 1: not the pending action)
	$(PY) scripts/merge_step.py ack --seq '$(SEQ)' $(if $(REFUSED),--refused) --next '$(or $(NEXT),90)'

merge-train-next: ## Step mode: wait up to WAIT=300 s for the run's next action and print it (decision 0238) (exit 3: none pending in that time)
	$(PY) scripts/merge_step.py next --wait '$(or $(WAIT),300)'

merge-flip-check: ## Whether a PR only sets the merge-train switch to lock (SPEC-0027 "The switch", decision 0157); prints the head sha to merge with expectedHeadSha: make merge-flip-check PR=123 (exit 0: a flip PR; 1: not one; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_flip.py check '$(PR)'

merge-train-stats: ## The merge train's metrics, the shadow agreement score and T-2991's evidence and load bar, from the GitHub API (SPEC-0027 "Metrics"): make merge-train-stats [SINCE=2026-10-29] [STRANDED=1: only the stranded-PR line, fast]
	$(PY) scripts/merge_train_stats.py $(if $(SINCE),--since '$(SINCE)') $(if $(STRANDED),--stranded)

merge-mode: ## The merge-train mode on a freshly fetched origin/main: lock, shadow or train (anything else reads as lock; SPEC-0027); `train (stop pending: #n)` while a valid merge-train-stop label is on PR n
	$(PY) scripts/merge_train.py mode
