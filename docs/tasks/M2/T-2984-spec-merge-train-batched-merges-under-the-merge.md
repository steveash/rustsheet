---
id: T-2984
title: "Spec: merge train — batched merges under the merge lock"
milestone: M2
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A reviewed spec for a batching merger (a "merge train"): workers hand a green, approved PR over
with a readiness marker and move on, and one merger tests several ready PRs together under the
merge lock and squash-merges them in order. Merge throughput then stops being one CI run per PR,
waiters stop starving, and sessions stop ending with finished PRs stranded. The spec also covers a
first, independent slice: a fair (FIFO) merge lock with a `wait` command. The owner approved
writing this spec (2026-10-28).

## Context
- Over the 48 h before 2026-10-25: 182 PRs merged (3.8/h, peaks of 7/h). The merge lock (decision
  0075) was held 60% of the time, median hold 8.1 min (about one CI run), median idle gap 0.7 min
  (saturated at peak). Median first-green to merge was 24 min, p90 135 min. 37 lock takeovers.
- A session can re-acquire the lock for its next PR at once, so waiters starve (30–40 min waits
  for #1860 and #2000 while one session chained #1859 then #1865). Workers end sessions with PRs
  stranded unmerged (e.g. #1826) because the budget ran out while they waited for the lock.
- PR CI is now about 4.9 min wall (decisions 0122, 0125, T-2755), so a lock hold is about 6 min:
  a ceiling of about 10 merges/h, serial. Every PR still pays at least one extra CI run under the
  lock, and `main` moving during that run forces repeat rounds (#1866 needed 3).
- Constraints: personal-account repository, so no GitHub merge queue (decision 0053 is dormant).
  Agents may not create, modify or delete routines. The `main` ruleset has no required status
  checks yet.

## Acceptance criteria
- [x] `docs/specs/SPEC-0027-merge-train.md` follows `docs/specs/TEMPLATE.md` and decides, with
      rejected alternatives: the readiness handoff (marker, who marks, how it is verified and
      invalidated); the merger (who runs it, lock ownership, the batch algorithm, how the train
      gets CI and the ci.yml change, why the merged `main` tree equals the tested train tree,
      bisection and ejection, crash and timeout recovery, conflicts, id collisions, docs-only
      PRs, orphaned PRs, required checks and auto-merge); a FIFO merge lock with `wait`; the
      decision amendments (0028, 0075) and their governance; a rollout in small reversible
      slices with metrics and a rollback plan; the full prompt text of the merger routine.
- [x] The rollout slices are filed with `make new-task`, each linking the spec, with `depends_on`.
- [x] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`,
      `check_governance.py origin/main`, `make typos` and the scripts unit tests pass.
- [ ] Reviewers approve (architecture for the spec). This box is ticked at merge, once every
      reviewer approves the final head.

## Out of scope
- Any change to `scripts/`, `.github/`, `.claude/`, `docs/process/` or the routines: those land
  with the rollout slices, each with its own decision record.
- Moving the repository to an organization or turning on the merge queue (decision 0053).

## Outcome
SPEC-0027 (`docs/specs/SPEC-0027-merge-train.md`, revision 3 after two review rounds)
designs the merge train:
- a fair, validated FIFO merge lock with `wait`, as the first, independent slice;
- a readiness marker that workers set before moving on. Its verdicts are an attestation, not a
  verification (owner decision); only markers by authors with write permission count;
- an owner-created merger routine, every 30 min, that polls for a 40-min window, plus a stall
  fallback;
- trains of up to 5 PRs on `merge-train/current`, squash-merged in order, with mandatory pre- and
  post-squash tree checks that detect (not prevent) a differing `main` and revert first;
- a binary search for the first red prefix;
- shadow mode on `merge-train/shadow`;
- the CI trigger, and a fix (T-2988) for docs-only PRs that are behind `main` being classified as code;
- an emergency switch to `lock` that any agent can merge;
- metrics with daily thresholds, and rollback.

No decision record lands here: the spec changes no protected path, and each slice carries its
own record. Follow-ups, in rollout order:
- T-2985: fair lock;
- T-2993: GitHub client and write probe;
- T-2986: readiness marker;
- T-2988: CI on train branches;
- T-2987: builder and dry run;
- T-2994: merge stage for green trains, with the revert path;
- T-2998: red trains (bisection, ejection);
- T-2995: shadow and stats;
- T-2996: policy record and routine prompts;
- T-2989: skills and process, `shadow`;
- T-2991: `train`, blocked on the owner's routine;
- T-2990: one-week measurement;
- T-2997 (P3): v2 verdict records.

Open questions for the owner are at the end of the spec. No eval delta.
