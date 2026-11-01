# 0241 — make hygiene runs the scripts' test modules in parallel

- **Status:** accepted (2026-11-01, owner request after a CI investigation, T-3705). It changes
  how the scripts' unit tests run, not which tests run.
- **Context:**
  - `make hygiene` is the critical path of every docs-only PR. In run 41288141328 the docs-only
    run took 2.0 min, and its hygiene job 116 s, 72 s of it in `make hygiene`. In heavy PR runs
    `make hygiene` took p50 104 s (2026-11-01).
  - Profiled locally on 2026-11-01, on a machine busy with other jobs:
    - The nine ratchet scripts take about 10 s together (`check_duplication.py` 6 s, the rest
      under 1.2 s each).
    - Almost all the rest is `unittest discover` over `scripts/`: 28 modules, 464 tests, 455 s
      serial.
    - The merge-train modules dominate: `test_merge_train_red` 142 s, `test_merge_train` 115 s,
      `test_merge_ready` 61 s, `test_merge_train_shadow` 42 s, `test_merge_lock` 35 s.
    - The tests build their git repositories under temporary directories, so modules do not
      share state.
- **Decision:**
  - `make hygiene` runs each `scripts/test_*.py` as its own `python3 -m unittest discover -s
    scripts -p <file> -q`, with one process per CPU (`getconf _NPROCESSORS_ONLN`), largest file
    first (`ls -S`) so the long modules start early.
  - Each module runs exactly as `discover` ran it: from the repository root, with `scripts/` on
    the path. `xargs` exits non-zero when any module fails, so the target fails as before.
- **Measured (local, 4 cores, same busy machine):** 464 tests in 28 modules, all OK. 455 s serial,
  144 s in parallel. The bound is the slowest module. CI (run 41301094082): `make hygiene` 46 s against a p50 of 104 s.
- **Consequences:**
  - A test module must not depend on another module running first or on shared fixed paths. The
    suite already holds to this; a new module that breaks it shows up as a failure here.
  - Output from parallel modules can interleave. A failing module still prints its own traceback,
    and the target's exit status is what gates.
  - Splitting `test_merge_train_red` would lower the bound further; that is not needed now.
