# 0041 — Eval runs under a crash-evidence wrapper (eval_guard.sh)

- **Status:** accepted (2026-10-08, T-1118)
- **Context:** A steward `make eval-baseline` died once from SIGSEGV (2026-10-06). It did not
  reproduce in 80 runs (T-0183), and make printed only "Segmentation fault", so the cause is
  unknown. The next crash is the only evidence we will get. Worker thread names (`case:<id>`) are
  cut to 15 bytes by Linux, so even a debugger backtrace cannot name the case.
- **Decision:**
  - `make eval`, `eval-check` and `eval-baseline` run `rustsheet-eval` through
    `scripts/eval_guard.sh`.
  - The wrapper runs the command under `gdb -batch` when gdb can trace it, so a crash prints every
    thread's backtrace. Otherwise it runs the command directly and names the signal.
  - Either way, it lists the cases that started but never ended, read from an opt-in case log
    (`RUSTSHEET_EVAL_CASE_LOG`, `crates/rustsheet-eval/src/case_log.rs`).
  - Exit codes pass through unchanged, and a signal still exits non-zero.
- **Consequences:**
  - Gates are unchanged: the same command and exit status, with about 1.5 s of gdb start-up and two
    or three gdb status lines in the output.
  - `RUSTSHEET_EVAL_GDB=0` opts out of gdb.
  - Direct `rustsheet-eval` calls (tools/, `eval-show`) are not wrapped.
  - If the crash recurs, file a task with the printed backtrace and case ids.
