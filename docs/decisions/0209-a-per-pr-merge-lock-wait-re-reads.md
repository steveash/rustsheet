# 0209 — A per-PR merge-lock wait re-reads the mode on every poll

- **Status:** accepted (2026-10-31, T-3515)
- **Context:** decision 0204 refuses a per-PR lock claim in `train` mode, but `wait` checked the
  mode once, at its start. After the re-flip to `train` (decision 0208), six per-PR waiters that
  had queued under `lock` kept their places; when the lock was freed for the train, the head
  waiter (#2624) took it first.
- **Decision:** `merge_lock.wait` asks `train_mode_refusal` before every poll for a `--pr` wait.
  A refusal ends the wait with exit 5, as at the start, and takes the waiter out of the queue at
  once instead of leaving a dead entry at its head for `WAITER_TTL`.
- **Consequences:** every per-PR wait poll costs one `git fetch` of `main` and, in `train` mode,
  one PR read. A holder that already has the lock when the mode switches still finishes.
