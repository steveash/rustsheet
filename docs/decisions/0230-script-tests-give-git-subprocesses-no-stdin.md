# 0230 — Script tests give git subprocesses no stdin

- **Status:** accepted (2026-11-01, T-3694)
- **Context:** on 2026-11-01 a local `make hygiene typos | tail -3` hung for 30 minutes at load 0.
  `scripts/test_id_registry.py`'s `run` helper called `git mktree`, which reads tree entries from stdin, and
  the helper inherited the caller's stdin, an open pipe. CI's stdin is empty, so the test passed there. The
  production helper (`merge_lock.git`, used by `id_registry.py`) already passes `stdin=subprocess.DEVNULL`.
  Reproduction: `sleep 60 | timeout 40 python3 -m unittest scripts.test_id_registry -q` timed out (exit 124)
  before the fix, and ran 9 tests in 1.9 s after it.
- **Decision:** a test under `scripts/` that runs git (or any command that may read stdin) passes
  `stdin=subprocess.DEVNULL`, as the production helpers do. T-3694 fixes the one helper that ran such a command
  (`git mktree`). The change only removes a hang; it weakens no gate and no assertion.
- **Consequences:** `make hygiene` and `make check` finish whatever stdin the caller gives them; the full script
  suite (464 tests) passed with stdin held open as a pipe. A new script-test helper copies the DEVNULL argument.
