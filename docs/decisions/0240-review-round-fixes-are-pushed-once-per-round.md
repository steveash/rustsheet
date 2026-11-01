# 0240 — Review-round fixes are pushed once per round

- **Status:** accepted (2026-11-01, owner request after a CI investigation, T-3703). It refines
  the `review` skill (step 4) and work-loop step 5. The review-round limit and the panel
  (decision 0236) are unchanged.
- **Context:**
  - Over the 100 CI runs of 2026-11-01 (05:53-09:16Z), 20 of 78 PR runs were cancelled because a
    newer push to the same branch replaced them. Together they had run about 77 runner-minutes.
  - PRs averaged 1.9 runs, and one had 5.
  - The review skill posts each reviewer's verdict record as it returns, and nothing said when
    to push fixes. So a worker could push after the first report while other reviewers still
    ran:
    - every push restarts the full CI run (about 6-10 min);
    - a reviewer still running reviews a head that no longer counts (its record names the old
      head, decision 0178), so it runs again.
- **Decision:** a worker pushes a review round's fixes once.
  - It waits until every reviewer of the round has reported, fixes all of the round's findings
    in local commits, and runs `make check`.
  - Then it pushes once and runs the next round on that head.
- **Consequences:**
  - A round costs one CI run and one set of reviewer runs. Verdict records still name the head
    they reviewed, so nothing changes in `make mark-ready` or the merger.
  - Fixes outside review are unaffected: a red-CI fix, a conflict resolution or an ejection fix
    is still pushed when ready (steward skill).
  - The effect can be read from the share of cancelled PR runs and the runs per PR (both
    measured above), for example by the steward's spot-audit.
