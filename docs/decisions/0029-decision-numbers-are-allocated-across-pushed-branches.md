# 0029 — Decision numbers are allocated across pushed branches

- **Status:** accepted (2026-10-05, T-0038)
- **Context:** decision records were numbered by hand ("highest on main + 1"), so two PRs written
  in parallel took the same number (0021, T-0032 and T-0033, 2026-10-04). `check_decisions.py`
  (decision 0023) only detects the clash, and two PRs green against the same old main can both
  merge. Task ids had the same problem, solved by `make new-task` (decision 0016).
- **Decision:** create records with `make new-decision TITLE="..."` (`scripts/new_decision.py`).
  It fetches origin and takes one above the highest number on the working tree and on every
  `origin/*` branch, through the same branch scan as `make new-task` (`tasks.git_branch_names`)
  and the record-name regex of `check_decisions.py`. Without git or a network it warns and uses
  the working tree. Push the record with the first commit on the branch so other sessions see the
  number.
- **Consequences:** a number already pushed by another session is never reused. A session that
  allocates before pushing can still race another one, so `check_decisions.py` and the merge-time
  up-to-date check (decision 0028) remain the guarantee. Task ids and decision numbers now follow
  one allocation rule.
