#!/usr/bin/env bash
# Runs a rustsheet-eval command so that a death by signal leaves evidence (T-1118, decision 0041).
#
# The 2026-10-06 steward run ended in a SIGSEGV that never reproduced (T-0183) and left only
# make's "Segmentation fault". Under this wrapper the same crash prints:
#   * a backtrace of every thread, when gdb can trace the process (it runs the command under
#     `gdb -batch`; exit codes pass through, a signal exits non-zero);
#   * the signal name, when gdb is missing or cannot trace (ptrace denied);
#   * the cases that had started but not ended, from the RUSTSHEET_EVAL_CASE_LOG file the eval
#     writes (crates/rustsheet-eval/src/case_log.rs).
# gdb adds a few status lines ("[Inferior 1 (process N) exited normally]"); exit codes are what
# gates read. Set RUSTSHEET_EVAL_GDB=0 to skip gdb.
set -u

if [ "$#" -eq 0 ]; then
    echo "usage: $0 <rustsheet-eval> [args...]" >&2
    exit 2
fi

case_log=$(mktemp "${TMPDIR:-/tmp}/rustsheet-eval-cases.XXXXXX") || exit 2
trap 'rm -f "$case_log"' EXIT
# Interrupted runs still remove the log (the EXIT trap runs on these exits).
trap 'exit 130' INT
trap 'exit 143' TERM
export RUSTSHEET_EVAL_CASE_LOG="$case_log"

use_gdb=0
if [ "${RUSTSHEET_EVAL_GDB:-1}" != 0 ] && command -v gdb >/dev/null 2>&1 \
    && gdb -q -batch -nx -return-child-result -ex "set debuginfod enabled off" -ex run \
        --args true >/dev/null 2>&1; then
    use_gdb=1
fi

if [ "$use_gdb" = 1 ]; then
    # `thread apply all bt` prints nothing once the program has exited normally.
    gdb -q -batch -nx -return-child-result \
        -ex "set debuginfod enabled off" -ex "set pagination off" -ex "set print thread-events off" \
        -ex "set print inferior-events off" -ex "handle SIGPIPE nostop noprint pass" \
        -ex run -ex "thread apply all bt 40" --args "$@"
    status=$?
else
    "$@"
    status=$?
    if [ "$status" -gt 128 ]; then
        echo "eval_guard: $1 died from signal SIG$(kill -l $((status - 128)) 2>/dev/null || echo $((status - 128)))" >&2
    fi
fi

# A run that finished, pass or fail, ended every case it started; anything left was running
# when the process died. A timed-out case's worker is never joined, so it is listed too.
unfinished=$(awk -F'\t' '$1 == "start" { s[$2] = "" } $1 == "timeout" { s[$2] = " (timed out, worker not joined)" }
    $1 == "end" { delete s[$2] } END { for (c in s) print "  " c s[c] }' "$case_log" | sort)
if [ "$status" -ne 0 ] && [ -n "$unfinished" ]; then
    echo "eval_guard: exit status $status; cases started but not ended:" >&2
    echo "$unfinished" >&2
fi
exit "$status"
