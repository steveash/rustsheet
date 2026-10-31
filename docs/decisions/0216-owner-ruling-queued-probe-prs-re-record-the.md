# 0216 — Owner ruling: queued probe PRs re-record the visible-parity lock for their new coverage

- **Status:** accepted (2026-10-31, owner ruling in session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd: "Go ahead with 1 and 2 to fix the
  visible parity lock off count issue")
- **Context:**
  - `eval/baseline/visible-parity.json` is one repository-wide cap on visibly-off cells
    (decisions 0088, 0093). `make eval-baseline` raises it only when the count passes it, so a
    probe PR that adds visibly-off known failures while `main` has headroom leaves it as is.
  - On 2026-10-31 `main`'s headroom was used up (#2598's 222 F.DIST cells) and the steward
    lowered the lock to the exact count (2427). Queued probe PRs then failed the train's eval
    gate on visible parity alone, with every known-failure list matching: #2581 (+129 cells),
    #2600 (+108) and the PRs of #2564's batch. Once merged with `main`, their cases are no longer
    "new", so `make eval-baseline` refuses the rise, and every PR that does raise the one number
    conflicts with every other.
- **Decision:**
  - Until the per-case lock (the follow-up of this ruling) lands, a PR ejected for a visible-parity
    rise may be caught up by any session as follows:
    - merge `main`, taking `main`'s side of `visible-parity.json` and of the known-failure lists;
    - run `make eval-baseline`; it must refuse **only** the visible-parity rise, with zero
      `REGRESSION` lines in every suite's list;
    - the rise over `main`'s lock must not exceed the visibly-off cells of the cases the PR adds;
    - record the lock with `rustsheet-eval update-baseline --allow-regressions` (this decision is
      the one `--allow-regressions` requires) and check that the diff against the plain merge
      touches only `visible-parity.json` and the lists `make eval-baseline` re-records;
    - push, and re-mark the PR with its existing verdicts once CI is green.
  - A rise that the PR's own new cases do not account for is a regression and goes back to the
    PR's worker.
- **Consequences:** the lock rises by exactly the new coverage the merged probe PRs add; no
  existing cell may get worse under this ruling. It expires when the per-case lock merges.
