# 0215 — Task ids and decision numbers are claimed in an atomic id registry

- **Status:** accepted (2026-10-31, T-3543; the owner asked for it)
- **Context:**
  - `make new-task` and `make new-decision` took one above the highest id on the working tree and
    on every pushed branch (decisions 0016, 0029). An id was protected only once its branch was
    pushed, and sessions often worked for an hour before their first push.
  - On 2026-10-31 alone, the merge train ejected or renumbered PRs for T-3467, T-3476, T-3479,
    T-3396, 0197 and 0204. Two clashes were allocate-then-push races; one was a number picked by
    hand during a renumber.
- **Decision:**
  - The branch `id-registry` on origin holds the recent claims: an orphan commit with an empty
    tree whose message lists them, one per line (scripts/id_registry.py).
  - Both commands pick the id with their existing rule over everything taken plus the registry's
    claims, then push the registry with the new claim under `--force-with-lease` (merge_lock's
    CAS). A lost race re-reads and picks the next id. Each claim commit carries a nonce, because
    two identical commits would let the loser's push pass as "up to date".
  - The registry keeps the newest 400 claims plus the highest claim of each task hundred-block and
    the highest decision number, so pruning cannot let an id be reissued. Offline (`--no-fetch`) or when it cannot be
    reached, allocation falls back to the old scan with a warning; `tasks.py validate`,
    `check_decisions.py` and the train's id check still catch what slips through.
  - Ids are never picked by hand, including in a renumber.
- **Consequences:** an id is reserved the moment `make new-task` or `make new-decision` prints it,
  with no "push now to claim" step. Each allocation costs one fetch and one push of a tiny branch.
