---
id: T-3694
title: "test_id_registry's run helper gives git mktree no stdin, so the test hangs when stdin is an open pipe"
milestone: M0
lane: harness
status: done
priority: P2
size: S
depends_on: []
---

## Goal
`make hygiene` (and `make check`) never hang in `scripts/test_id_registry.py`, whatever the caller's stdin is.

## Context
- `seed` in scripts/test_id_registry.py builds an empty tree with `git mktree` through the file's `run` helper. That
  helper did not set `stdin`, and `git mktree` reads its tree entries from stdin. When `make hygiene` runs with
  stdin as an open pipe (an agent shell piping the output, for instance), the test blocks forever. In CI stdin is
  empty, so it passed there.
- Seen on 2026-11-01: a local `make hygiene typos | tail -3` sat for 30 minutes with load 0 on `git mktree`
  (cwd a temp clone `.../b`), under `python3 -m unittest discover -s scripts`.
- The production helpers already pass `stdin=subprocess.DEVNULL` (`merge_lock.git`, which id_registry.py uses).

## Acceptance criteria
- [x] Reproduced: `sleep 60 | timeout 40 python3 -m unittest scripts.test_id_registry -q` hit the timeout (exit
      124) before the fix.
- [x] With the fix, the same command runs 9 tests in about 2 s and passes.
- [x] Decision 0230 records the rule (scripts/ is a protected path).
- [x] `make check` green; reviewers approve.

## Out of scope
- The other `run(*args, cwd)` helpers in scripts/test_merge_*.py run no stdin-reading git command.

## Outcome
`run` in scripts/test_id_registry.py passes `stdin=subprocess.DEVNULL`, as the production `git` helpers do. Before
the fix, the reproduction above timed out (exit 124); after it, 9 tests pass in 1.9 s.
