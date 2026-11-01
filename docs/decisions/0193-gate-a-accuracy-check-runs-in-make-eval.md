# 0193 — Gate A accuracy check runs in make eval-gates

- **Status:** accepted (2026-10-31, T-3356). Implements SPEC-0029 "The check" and decision 0188 R2.
- **Context:** Decision 0188 R2 makes `make check` fail on a reference-table point outside Gate A's
  envelope, and SPEC-0029 puts that check in a new subcommand, `rustsheet-eval accuracy`, run by
  `make eval-gates` right after `check --evidence-floor`, so that `make check` and CI's
  `eval-gates` job both run it. The Makefile is a protected path (CLAUDE.md directive 2), so the
  wiring needs this record.
- **Decision:** the `eval-gates` target runs `$(EVAL_GUARDED) accuracy` as its second command,
  after `$(EVAL_GUARDED) check --evidence-floor`. A failing point, a stale known-failure line, an
  R3 mark without evidence or a table defect fails the target, and so `make check` and CI.
  No other target changes: `make eval`, `eval-check`, `eval-baseline` and the evidence floor never
  read `eval/accuracy/`.
- **Consequences:**
  - The gate adds a check; it weakens none. Until T-3358 commits real tables, `eval/accuracy/`
    holds only its manifest and empty lists, and the check passes with 0 points.
  - The check's failing behaviour is shown by a fixture under the harness crate's test data,
    not by a table under `eval/accuracy/` (T-3356 acceptance).
  - The check must stay under SPEC-0029's 30 s budget; the summary line prints its time.
