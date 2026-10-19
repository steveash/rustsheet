# 0028 — Merge only a head that contains the current main

- **Status:** accepted (2026-10-05, owner request). Amended by 0053: with the merge queue on, the
  queue gives this guarantee. Amended by 0075: merges are made under the merge lock, so the check
  passes on the first green CI after the lock is taken. It refines 0023 (a clash caught by the
  second PR's CI now also has to be caught at merge time) and the up-to-date clause of the steward
  merge policy, fits 0020 (merge `main` in, never rebase), and leaves 0016's allocator as best
  effort.
- **Context:** On 2026-10-05, steward PR #79 and T-0109 PR #81 each added a task numbered T-0129.
  Each PR's CI was green, because CI tests the PR merged with the `main` of its last push, and
  #81 was merged after #79 landed. GitHub reported `mergeable_state: clean`, which only means "no
  textual conflict": two new files with the same id never conflict. `tasks.py validate` then
  failed on `main` until #84 renumbered one of them. Decision numbers had the same race earlier
  (two 0021 records, from T-0032 and T-0033; decision 0023), and two open branches carry a decision 0026 right now. Allocation
  (`make new-task`, T-0038's `make new-decision`) only sees pushed branches, and agent sessions
  and subagents can allocate before pushing, so no allocator can make ids unique on its own.
  Gaps in the numbering are acceptable; duplicates are not.
- **Decision:** the guarantee lives at merge time.
  - The steward skill's merge policy check (c), "up to date with `main`", is now a mechanical
    step right before every merge: fetch `origin/main` and require
    `git merge-base --is-ancestor origin/main origin/<branch>`. Otherwise merge `main` in, run
    `make check`, push, and merge only after CI is green on that head. The same CI then runs
    `tasks.py validate` and `check_decisions.py` against the real merged tree, so a duplicate
    fails before it lands and the PR renumbers its own record.
  - Recommended to the owner: enable branch protection on `main` with the required CI checks and
    "Require branches to be up to date before merging", so GitHub itself refuses a stale merge.
    That covers every merger, including ones that skip the skill.
  - Allocation stays best effort: allocate with the tools, and push the file in the branch's
    first commit so other sessions see the number.
- **Consequences:** a PR whose base moved costs one more CI cycle before it can merge. A
  duplicate costs a renumber on the PR that merges second, never a red `main`. This record
  uses 0028 because two open branches already hold 0026. It leaves 0027 free on purpose, so
  whichever of them merges second can renumber to it.
