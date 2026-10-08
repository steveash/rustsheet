from __future__ import annotations

import os
import shutil
import subprocess
import unittest
from pathlib import Path

GUARD = Path(__file__).resolve().parent / "eval_guard.sh"

# Stands in for rustsheet-eval: logs cases like crates/rustsheet-eval/src/case_log.rs, then runs
# the tail command. Case b starts but never ends.
FAKE_EVAL = 'printf "start\\tsuite/a\\nstart\\tsuite/b\\nend\\tsuite/a\\n" >> "$RUSTSHEET_EVAL_CASE_LOG"; '


def guard(tail: str, gdb: str = "0") -> subprocess.CompletedProcess[str]:
    env = {**os.environ, "RUSTSHEET_EVAL_GDB": gdb}
    return subprocess.run(
        [str(GUARD), "sh", "-c", FAKE_EVAL + tail],
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )


class EvalGuard(unittest.TestCase):
    def test_signal_names_signal_and_unfinished_cases(self) -> None:
        run = guard("kill -ABRT $$")
        self.assertEqual(run.returncode, 134)
        self.assertIn("died from signal SIGABRT", run.stderr)
        self.assertIn("  suite/b", run.stderr)
        self.assertNotIn("suite/a", run.stderr)

    def test_exit_status_passes_through_without_listing(self) -> None:
        # A finished run (pass or fail) ended every case it started.
        for code in (0, 1, 3):
            run = guard(f'printf "end\\tsuite/b\\n" >> "$RUSTSHEET_EVAL_CASE_LOG"; exit {code}')
            self.assertEqual(run.returncode, code)
            self.assertNotIn("eval_guard", run.stderr)

    @unittest.skipUnless(shutil.which("gdb"), "gdb not installed")
    def test_gdb_mode_keeps_exit_status(self) -> None:
        # Falls back to a plain run when ptrace is denied; the status is the same either way.
        self.assertEqual(guard("exit 3", gdb="1").returncode, 3)
        self.assertEqual(guard("exit 0", gdb="1").returncode, 0)

    @unittest.skipUnless(shutil.which("gdb"), "gdb not installed")
    def test_gdb_mode_signal_fails_the_run(self) -> None:
        # The gate must stay fail-closed when gdb catches the crash.
        run = guard("kill -ABRT $$", gdb="1")
        self.assertNotEqual(run.returncode, 0)
        self.assertIn("  suite/b", run.stderr)


if __name__ == "__main__":
    unittest.main()
