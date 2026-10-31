# 0205 — Ejected PRs with baseline-only conflicts catch up without a new review

- **Status:** accepted (2026-10-31, T-3479; the owner asked for the fix)
- **Context:**
  - The merge train ejects a PR whose merge with `main` conflicts ("conflict with main"). Only
    the PR's worker could clear it: merge `main` in, re-mark, and run the reviewers again,
    because `make mark-ready CARRY=1` carried verdicts only over a clean merge.
  - Workers' sessions usually end before their PR merges, so ejected PRs stranded. On
    2026-10-31, seven PRs sat ejected for hours.
  - Most of these conflicts are only in `eval/baseline/`, the known-failure lists that almost
    every engine PR edits. The per-PR flow already resolves those mechanically: take `main`'s
    side, then `make eval-baseline` (decision 0093).
- **Decision:**
  - **Carry rule.** The carry check (`merge_ready.Repo.carry_problem`, SPEC-0027 "Carry-over")
    accepts a merge of `main` into the marked head whose tree is the plain merge of its parents
    except in `eval/baseline/`. A conflict or an edit anywhere else still breaks the carry. The
    eval gate, which marking requires green on the head, checks the lists against the engine
    exactly, so a wrong list cannot be marked.
  - **`make catch-up PR=<n>`** (scripts/merge_catchup.py), in the caller's checkout:
    - merges `origin/main` into the PR branch;
    - refuses any conflict outside `eval/baseline/`;
    - takes `main`'s side of the lists and runs `make eval-baseline`, which refuses regressions;
    - commits and pushes (never forced), then restores the checkout.
    Once CI is green, `make mark-ready PR=<n> CARRY=1` re-marks with the old verdicts. Any session
    may do this (steward skill, "Ejections").
- **Consequences:**
  - A baseline-only ejection is cleared in one catch-up and one CI run, with no new review and
    without the PR's original session.
  - A PR whose own diff touches `eval/` still needs excel-fidelity's verdict, which carries over
    with the others. A catch-up that re-records lists for a PR that had no eval change makes
    excel-fidelity required, so the carry is refused and the reviewers run again.
  - Conflicts in code, docs or task files still go to the PR's worker.
