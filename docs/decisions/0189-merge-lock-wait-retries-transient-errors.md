# 0189 — merge_lock wait retries transient errors

- **Status:** accepted (2026-10-31, T-3345). It amends 0145's `wait` rules (T-3010).
- **Context:** On 2026-10-30, with up to 15 waiters in the queue, `merge_lock.py wait` exited 2
  three times while its caller was still queued (#2371 twice, #2344 at the head of the queue).
  `wait` gave up after `GIT_FAILURES` (3) git errors in a row, and did not retry a
  `LockFormatError` at all, so a racing write or a few push or fetch failures under load ended
  the wait. A head waiter that dies holds the reservation for nobody until it is dropped
  (2 min unclaimed, 10 min unrefreshed), which stalls everyone behind it.
- **Decision:** `wait` sorts errors into permanent and transient.
  - **Permanent, exit 2 at once:** a bad holder id or PR number (checked before the first
    poll), and a queue that is full (`QUEUE_MAX` waiters) when the caller first tries to join.
  - **Transient, retried until the timeout:** git fetch and push errors (`GitError`), an
    unreadable marker (`LockFormatError`, a racing write), and a compare-and-swap lost
    `ATTEMPTS` times (already a plain busy poll).
    Each distinct message is printed. A queue that fills up after the caller joined does not
    end its wait.
  - **A marker that stays unreadable** for `UNREADABLE_POLLS` (5) polls in a row is persistent,
    not a race, and exits 2.

  Retries keep the normal 30 s poll interval and do not grow it. A head waiter must claim a
  freed lock within 2 min (0145), so a growing backoff would cost it its place. The waiter's
  queue entry is never removed on an error, so it keeps its place.
- **Consequences:** an exit 2 from `make merge-lock-wait` now means a real problem (bad
  arguments, a full queue at join, a broken marker), not load. The steward skill's "re-run the
  wait" on exit 2 still applies. `GIT_FAILURES` is gone. A git outage that lasts the whole
  timeout ends in exit 1 (timeout), not exit 2. Follow-up, optional: have `attempt` return a
  structured status so `wait` stops matching `busy_message` text.
