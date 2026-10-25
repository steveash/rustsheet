# 0095 — Tasks blocked at the review-round limit are re-reviewed by the opus worker

- **Status:** accepted (2026-10-21, T-2284). Refines decision 0048 (a task blocked on its branch
  stays claimed at any age). Amended by 0121: on resume, merge `origin/main` in only if it
  conflicts or collides.
- **Context:**
  - Work-loop step 5 allows three review rounds. After the third, the run sets `status: blocked`
    on the branch, marks the PR draft and moves on, usually with the last round's fixes pushed and
    never reviewed. Decision 0048 keeps such a branch claimed forever, both worker prompts skip
    `status: blocked` branches, and the steward merges only non-draft, non-blocked PRs, so no
    routine ever looked at them again.
  - On 2026-10-21 four PRs sat in this state (#1357, #1385, #1245, #1353); the owner unblocked and
    merged them by hand. One (#1245) was blocked only by the performance reviewer that decision
    0081 had since turned off.
  - Other blocks must not be reopened this way: time-boxed fits (decision 0056), missing
    dependencies or capabilities (decision 0013), the desktop oracle (decision 0039).
- **Decision:**
  - *Marker.* Work-loop step 5 starts the `## Blocked because` section of a task blocked only by
    the round limit with the line `Blocked because: review-round limit`. No other block uses it.
  - *Offer.* `make next-task` offers a task whose pushed branch carries that marker on a
    `status: blocked` task file, once the branch has had no commit for 2 h, to the opus tier
    only: after P0 tasks and handoffs, before other work, whatever its lane is doing. It prints
    the branch to resume.
  - *Resume* (work-loop step 5c). The opus run brings `origin/main` in, sets `status:
    in-progress`, replaces the marker with `Resumed after the review-round limit (step 5c).` and
    pushes that as its claim, then runs the full reviewer panel from round 1 on the current diff,
    with the usual three rounds.
  - *Once only.* If the resumed task hits the limit again, the run writes `Blocked because:
    review-round limit, twice`, which `next-task` does not offer. The steward reports it under
    "Needs owner".
  - *Visibility.* The steward lists every PR blocked at the review-round limit under open PRs in
    `docs/status.md` with its age.
- **Consequences:**
  - A round-limited branch costs one extra opus review cycle instead of sitting claimed until the
    owner notices. A second failure goes to the owner instead of looping.
  - The worker routine prompts (decision 0008) are not changed by this decision: `next-task`
    surfaces the task in step (1) or (3) of the opus prompt, and work-loop step 1 sends a task
    printed with `resume ... (work-loop step 5c)` to step 5c instead of a new branch. The owner
    may add the case to the opus prompt's step (1) later.
