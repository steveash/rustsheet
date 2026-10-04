# 0016 — Task ids are allocated by `make new-task`

- **Status:** accepted (2026-10-04; T-0017)
- **Context:**
  - Ids were picked by hand ("next free id"). On 2026-10-04 two pairs of parallel sessions picked the
    same ids twice (#47 reused T-0010/T-0011; #48 and #49 both created a T-0015). Each collision cost
    a renumber, a board regenerate, a `make check` and a CI cycle, and `validate` only caught it
    after the merge conflict.
  - Both collisions were between unmerged branches, so checking `main` alone would not have
    prevented them.
- **Decision:**
  - `scripts/tasks.py new` (`make new-task`) writes the task file from the template. Its id is the
    next one after the highest id used in the milestone's hundred-block (M0 → T-00xx, M1 → T-01xx,
    M9 → T-09xx), counting the working tree and the task files on every `origin/*` branch after a
    `git fetch`. Ids only grow; gaps are not reused.
  - The new file must be committed and pushed at once: the push is the claim other sessions see.
  - `validate` names both files of a duplicate and points at `make new-task`, and rejects files that
    still contain the template's `TODO(fill)` placeholders.
  - `scripts/test_tasks.py` (stdlib unittest) runs in `make hygiene`.
- **Consequences:**
  - A collision now needs two sessions to create a task in the same milestone within one
    fetch-and-push window, instead of within the lifetime of an unmerged PR. `validate` in CI still
    catches the rare remaining case.
