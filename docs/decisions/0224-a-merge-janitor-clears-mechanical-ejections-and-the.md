# 0224 — A merge janitor clears mechanical ejections, and the train escalates before it flips

- **Status:** accepted (2026-11-01, T-3630; owner ruling: the repository owner approved this design
  in session https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd, "yes i like this go ahead and
  build this too"). Amends 0050, 0157 and 0194, extends 0207's carry rule, and answers SPEC-0027
  open question 3.
- **Context:**
  - In `train` mode the merger ejects a PR for `conflict with main`, an id collision or `ci-red`,
    and only the PR's worker clears it. Workers' sessions usually end first, so ejected PRs sat for
    hours. `make catch-up` (0207) clears baseline-only conflicts, but only when a session runs it.
  - An id collision is mechanical: the PR renumbers its own task or decision file (0121 (2)).
    SPEC-0027 section 8 kept the merger from doing it: a renumber is a push after review, can race
    a live session, and is not mechanical (references change too). Open question 3 asked the owner.
  - A stranded ready PR triggered the emergency flip to `lock` (0157, 0194), even when a single
    merger pass would have merged it. Liveness problems turned the train off; correctness never
    needed that.
  - The orphan clock (0050: 24 h open, 2 h idle, `agent/*` branches only) left ejected `claude/*`
    branches and day-old idle PRs alone.
- **Decision:**
  1. **`make renumber PR=<n>`** (scripts/merge_renumber.py, scripts/id_renumber.py). In a clean
     checkout, for an open, same-repository PR based on `main`: find the task and decision files
     the PR adds whose id `main` also has; refuse when there are none or the merge with `main`
     conflicts outside the eval-gated lists; reserve a fresh id for each in the id registry (0215:
     a task id in the task's own milestone, a decision number; never by hand; refuse when the
     registry cannot reserve); move each file to its new id and replace the old id with the new
     one **only** in files the PR added (so the task's `id:` and the record's `# NNNN —` title
     follow) and in the lines the PR added to files it modified (`git diff -U0`, pinned to
     myers); merge `main` in without committing (baseline-list conflicts resolved and re-recorded
     as catch-up does); commit one merge commit whose parents are the PR head and `main`; push,
     never forced; restore the checkout. A line `main` owns keeps the old id: it names `main`'s file.
  2. **The carry rule, extended** (`merge_ready.Repo.carry_problem`). `make mark-ready CARRY=1`
     also accepts a merge of `main` into the marked head whose tree equals, with 0207's and 0219's
     allowances for the eval-gated lists, the merge of `main` with the marked head renumbered
     exactly as `make renumber` renumbers it (`id_renumber.renumbered_merge`). The renames the
     head shows against the plain merge give the old and new ids. Each old id must be the id of
     exactly one file the PR added, and each new id must be of the same kind and on neither `main`
     nor the PR head. The tool and the rule build the renumbered tree with the same function, and
     the rule compares whole trees, so any other edit is refused. That includes a substituted line
     `main` owns, an edit in a renamed file, a rename of `main`'s file and a rename onto an id `main` has.
  3. **`make janitor`** (scripts/merge_janitor.py; `DRY_RUN=1` lists actions only), run by the
     merger routine after every pass, in `train` mode only. Idle means time since the PR's current
     head was pushed (the activity API). For open, non-draft, same-repository PRs:
     - ejected for `conflict with main` with conflicts only in the eval-gated lists, idle 1 h:
       `make catch-up`; ejected for an id collision, idle 1 h: `make renumber`. It comments a
       `merge-janitor v1` note naming the pushed head and does not wait: a later run marks the PR
       with `CARRY` once CI on that head is green;
     - ejected for `ci-red` or a real conflict, or after a refused step or red CI on its own push,
       idle 4 h: one comment per head that the PR is orphaned and needs an opus run or the owner;
     - a marker on an older head, idle 2 h: re-mark with `CARRY` when the carry rule accepts the
       head and CI is green; otherwise remove `ready-to-merge` (as V2 does) and leave it to the
       orphan rule.
  4. **Escalation replaces the flip for a stranded PR** (amends 0157 and 0194). A PR marked ready
     on its current head, not ejected and unmerged 2 h later is stranded. Ejected PRs, PRs waiting
     on their worker and stale-marker PRs never count. Then:
     1. the steward runs one `make merge-train TRAIN=1` pass and `make janitor`, and files a P0
        with the cause; no flip;
     2. if nothing has merged 6 h later, with ready PRs waiting and the steward's own pass failing,
        it files a P0 for the owner, and flips to `lock` only with the owner's approval;
     3. the immediate emergency flip is kept only for correctness failures: `main` got a tree no
        train tested (a revert PR, or a P0 tree mismatch), or a train merged something CI
        rejected. The other daily thresholds (p90 latency, CI runs per train) escalate like a
        stranded PR (`make merge-train-stats` prints `-> escalate`, and `-> flip to lock` only for
        a tree-check failure).
  5. **Orphaned PRs in `train` mode** (amends 0050 and 0157): 6 h idle replaces "24 h open", and
     `claude/*` branches count as well as `agent/*`. The steward still runs the reviewers itself
     and marks the PR ready.
- **Consequences:**
  - Ejections for baseline-only conflicts and id collisions clear within about an hour plus one
    CI run, with no reviewer run and no live session. The janitor pushes only to idle branches,
    never forced, so a live session's push wins and a push that raced it is refused.
  - After a renumber, lines `main` owns keep the old id by design: they name `main`'s file.
  - The merger routine prompt gains the janitor step (docs/process/routines.md); the owner updates
    the routine (decision 0008).
  - The flip to `lock` becomes rare. Liveness problems reach the owner as P0s, not as a mode change.
