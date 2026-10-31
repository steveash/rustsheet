# SPEC-0027 — Merge train: batched merges under the merge lock

- **Status:** draft (revision 4: addresses review rounds 1 on 4802b41, 2 on a622e22 and 3 on 746cb01)
- **Milestone / lane:** M2 / harness (policy and process slices: quality)
- **Tasks:**
  - T-2984: this spec.
  - Rollout: T-2985 (fair lock), T-3001 (GitHub client and write probe), T-2986 (readiness
    marker), T-2988 (CI on train branches), T-2987 (train builder and dry run), T-2994 (merge
    stage for green trains), T-2998 (red trains: bisection and ejection), T-2995 (shadow mode
    and stats), T-2996 (policy record, routine prompts, emergency flip check), T-2989 (skills
    and process, switch to `shadow`), T-2991 (switch to `train`), T-2990 (one-week
    measurement).
  - v2: T-2997 (verdict records bound to the head sha).

## Problem
Every merge to `main` happens under the merge lock (decision 0075). The PR's own worker takes the
lock, merges `origin/main` in (decision 0121), waits for one more CI run, then squash-merges.
Merges are therefore serial, at one CI run each, and the worker waits through that run.

Measured over the 48 h before 2026-10-25:

| measure | value |
|---|---|
| PRs merged | 182 (3.8/h mean, peaks of 7/h) |
| share of time the lock was held | 60% overall, saturated at peak (median idle gap 0.7 min) |
| median lock hold | 8.1 min, about one CI run |
| "merge main" commits | 383, of which 190 were made before the lock and wasted (fixed by decision 0121) |
| first green CI → merge | median 24 min, p90 135 min |
| lock takeovers (stale or finished holder) | 37 |
| CI runs on "merge main" heads (`pull_request`, 2026-10-23 to 2026-10-25 UTC) | 215: 178 success, 18 failure, 19 cancelled |

The design causes three failures:

1. **Starvation.** A session that just released the lock can take it again at once for its next
   PR, ahead of sessions that have been retrying for minutes. #1860 and #2000 (T-2755) each waited
   30–40 min while one session chained #1859 and then #1865.
2. **Stranded PRs.** A worker's session ends on its time budget (decision 0025). If it was still
   waiting for the lock, it ends with a green, approved PR unmerged (for example #1826). The PR
   then waits 24 h for the steward's orphaned-PR rule (decision 0050).
3. **A ceiling and a CI tax.** PR CI takes about 4.9 min wall (decisions 0122 and 0125, T-2755).
   A hold therefore lasts about 6 min, and the serial ceiling is about 10 merges/h, close to the
   observed peak. Every PR pays at least one extra CI run under the lock. When `main` moves during
   that run, the PR pays again: #1866 (T-2761) needed 3 rounds, one of them for a task-id
   collision.

**Per-PR breakage rate.** The last row of the table puts the share of red runs on "merge main"
heads at p ≈ 18 / 196 ≈ 0.09 (cancelled runs excluded). That window holds both pre-lock and
under-lock merges, and conflict resolutions too, so 0.09 is an upper bound for a clean
under-lock merge. A train of n PRs is red with probability 1 − (1 − p)^n:

| n | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| P(red train), p = 0.09 | 9% | 17% | 25% | 31% | 38% |

GitHub's merge queue would solve this, but a repository owned by a personal account cannot turn
it on (decision 0053 is dormant; see 0075).

## Goals / non-goals
Goals:
1. A worker hands a green, approved PR over and moves on. It never waits for the lock to merge.
2. Several ready PRs share one CI run. The merger tests them merged together on top of `main`,
   then squash-merges them in order. Any way in which `main` could end up with a tree other than
   the tested one is detected at once, and a revert PR is opened for review (section 4, step 7).
3. The decision 0028 guarantee holds: nothing lands on `main` unless the tree it produces passed
   CI together with every PR merged before it. A duplicate task id or decision number still fails
   before it lands.
4. A failing PR is found and ejected without blocking the rest of its batch.
5. The lock is fair: waiters are served first-in, first-out, and a holder cannot jump the queue.
   This ships first and does not depend on the rest.
6. Every slice is reversible. The old per-PR flow stays available behind one switch, and turning
   the train off is a single, cheap action.

Non-goals:
- Moving the repository to an organization or turning on the merge queue. If that happens, 0053
  applies again and the merge train is retired along with the lock.
- Changing what CI runs on a PR, the review panel, or conditions (a) and (b) of the merge policy.
- Verifying reviewer verdicts cryptographically or per reviewer. That is v1's accepted residual
  risk (section 2, "Trust"), and the v2 follow-up is T-2997.
- Speculative parallel trains (Alternatives).

## Excel behavior to match
None. This is a process spec: it changes how PRs reach `main`, not what the engine computes.

## Design

### Overview
```
worker:  CI green + reviewers APPROVE ──▶ make mark-ready PR=n ──▶ next task (no waiting)
                                              │  label ready-to-merge + marker comment (head sha, verdicts)
merger:  routine every 30 min, each run polls for 40 min
          ──▶ ready PRs? ──▶ wait for lock (FIFO) ──▶ collect (FIFO, ≤ 5)
          ──▶ build merge-train/current = origin/main + each PR head merged in order
          ──▶ local id checks ──▶ push ──▶ CI on that exact sha
          ──▶ green: per PR: check GitHub's test merge → squash (expectedHeadSha) → check main
          ──▶ red:   binary-search the first red prefix, merge the green prefix, eject the culprit
          ──▶ release
```

**What the train buys at today's breakage rate.** At p ≈ 0.09 the main wins are three:
- **latency:** simulated p90 from marker to merge is about 13 min, against 135 min today from
  first green to merge;
- **no waiting** in worker sessions;
- **no stranded PRs.**

CI cost stays roughly where it is: about 1.05 merge-stage CI runs per merged PR at p ≈ 0.09 and
3.8/h (1.08 with the final rule of section 4), against about 1.05 plus repeat rounds today. It
falls only when p is small, to about 0.85 at p = 0. Batches stay small because the merger polls
(about 1.2 PRs at 3.8/h), so few PRs share a run. The figures come from the performance review's
simulations and a rerun with this spec's final rule (section 3, "Expected behaviour").

**The switch.** `scripts/merge_train.toml` holds `mode = "lock" | "shadow" | "train"`. It is
always read from a freshly fetched `origin/main` (`git fetch origin main && git show
origin/main:scripts/merge_train.toml`, as `make merge-mode` does), never from the local checkout,
so a branch cannot switch its own flow. **The read fails closed.** A missing file, a TOML parse
error, a duplicate key, a missing or non-string `mode`, or an unknown value all read as `lock`,
for workers and merger alike. The modes:

| mode | workers | merger |
|---|---|---|
| `lock` | today's per-PR flow under the fair lock | stops at once |
| `shadow` | per-PR flow as in `lock`; also run `mark-ready` (harmless) so the shadow merger has candidates | builds and tests trains on `merge-train/shadow`, takes no lock, merges nothing (section 4, "Shadow") |
| `train` | `mark-ready`, then move on | merges (section 4) |

Changing the mode (decision record of T-2996):
- **To `lock`** (the emergency off-switch): any agent, in any mode. A **flip PR** is a PR whose
  only change sets `mode = "lock"`. It may be squash-merged at once, by any agent, with no
  reviewers, under the fair lock. Waiving the protected-path review is deliberate and works in
  one direction only: turning the train off can never make a merge less safe than today's flow.
  The rule is mechanical (T-2996):
  - **The diff.** `git diff --name-status --no-renames <merge-base> <head>` is exactly
    `M scripts/merge_train.toml`. The file mode is `100644` on both sides, so no symlink and no
    type change.
  - **The blob rule.** Both versions of the file (at the merge base and at the head) parse with
    `tomllib`, which rejects duplicate keys. Their parsed documents are identical, except that the
    top-level `mode` at the head is the string `"lock"`. A nested `mode`, a comment trick or a
    second key therefore fails the rule.
  - **`check_governance.py`** waives the decision-record requirement only for a diff that passes
    this rule. Its other checks (test deletion, probe tags) still run.
  - **`make merge-flip-check PR=<n>`** applies the same rule through the GitHub files API, which
    must list exactly one file and its blobs, and prints the head sha it checked. The merging
    agent runs it, then merges with `expectedHeadSha` set to that sha.
  - **The P0 task** explaining the flip goes in a separate task-only PR, so the flip PR stays one
    file.
  - **The stop label.** The flipping agent labels the flip PR `merge-train-stop`. The label means
    "a flip to `lock` is pending". It is honoured only on an **open PR that currently passes
    `make merge-flip-check`**, and a train that finds one stops at its next checkpoint
    (section 4). Elsewhere it is ignored, and the merger removes it with a comment (`stop label
    removed: not a valid flip PR`).
    - On a personal-account repository, labels can be set only by accounts with write access (the
      owner and collaborators; there is no triage role), so an outsider cannot raise a stop.
    - A stale label clears itself: once the flip PR is merged or closed it is no longer open, so
      it no longer counts, and the merger removes the label from any PR that fails the check.
    - `make merge-mode` prints the mode and, when one exists, the honoured stop label and its PR
      (`train (stop pending: #n)`).
  - **CI and delay.** The flip PR changes `scripts/`, so it runs the full CI, about 5 min, not the
    docs-only path. In `train` mode the lock is held only by trains, and a live train stops within
    about 1 min of seeing the stop label. Worst-case emergency delay: about 5 min of CI, which
    overlaps the train, plus about 1 min for the train to stop, plus the merge. That is under
    10 min.
- **To `shadow` or `train`:** a normal, fully gated PR (reviewers, a decision record, architecture
  and security approval). The steps are T-2989 and T-2991.

### 1. A fair merge lock (first slice, T-2985)
The lock is still one commit on the `merge-lock` branch, written only by `scripts/merge_lock.py`
with `--force-with-lease`. That one commit holds both the holder and the queue, so a single
compare-and-swap (CAS) updates both. Two refs could not be swapped together atomically.

**Format** (`version: 2`; a commit without `version` is read as version 1 with an empty queue):
```
merge-lock

version: 2
holder: <id>                        # absent when free
pr: <n> | train
prs: 2101 2104 2110                 # train only: the batch's PRs still unmerged
renewed: <unix>                     # TTL clock (waiter writes rewrite the commit, so the commit time is not used)
state: free                         # only when free
freed: <unix>                       # only when free: when it was released
waiter: <id> <n|train> <registered unix> <seen unix>
waiter: ...                         # queue order, at most 20 lines
```

**Validation, on write and on parse.**
- Ids match `[A-Za-z0-9._-]{1,64}`, PR numbers are positive integers, and the only non-numeric
  `pr` value is `train`.
- `prs` holds at most `BATCH_MAX` (5) numbers.
- A malformed line is dropped, never rejected wholesale. A newline inside a value cannot survive,
  because the regexes reject it on write.
- **Fail closed on the holder.** A commit that is not `state: free` but has a missing or malformed
  `holder` reads as **held** by `unknown`. It is never read as free, and it goes stale after the
  TTL like any other.
- **Timestamps.** A missing or malformed `renewed` reads as the lock commit's committer time. A
  timestamp more than 60 s after the commit's committer time is clamped to that committer time. It
  is never clamped to "now" at read time, which would keep a bad lock fresh forever. A waiter's
  `seen` or `registered` that is out of range reads as 0, so the waiter has expired.
- The queue is capped at 20 waiters. A 21st registration exits 1 (`queue full`).

**Clocks.** Every age is computed with the reader's clock against timestamps written with the
writer's. A skew of under a minute, which is normal for NTP-synced runners, shifts the TTLs by
that much and nothing else. A clock that runs ahead cannot extend a lock past the committer-time
clamp above.

**`acquire --pr N`** (or `--train ID --prs ...`):
1. Read the lock and prune:
   - drop waiters whose `seen` is more than 10 min old (`WAITER_TTL`);
   - when the lock is free and its `freed` is more than 2 min old, drop the head waiter. A live
     head polls every 30 s and claims a free lock within that time, so a head that has not
     claimed it in 2 min has stopped waiting;
   - when the lock is free, stale or finished and the caller is not the head, drop the head
     waiter if its PR is finished. This uses decision 0097's API read, made once per PR per
     `acquire` and, across `wait`'s polls, only when the head changed or on every 10th poll, so a
     waiter makes at most about 12 reads an hour per PR. Unauthenticated access allows 60 reads
     an hour.
2. If the caller holds the lock, renew it (`renewed` = now; `prs` replaced when given) and exit 0.
   A renewal never reorders the queue.
3. If another agent holds the lock and it is neither stale (30 min since `renewed`) nor finished,
   add the caller at the tail or refresh its `seen`. The refresh is written only when `seen` is
   3 min old, to limit pushes. Exit 1: `busy: <holder> for PR <pr>; you are waiter k of n`.
4. If the lock is free, stale or finished, take it only when the queue is empty or the caller is
   at its head, and remove the caller from the queue. Otherwise register or refresh the caller and
   exit 1: `reserved for <head>`.

**Finished train locks.** A `pr: train` lock is finished only when `prs` is non-empty and every
PR in it is merged or closed. A train lock with an empty `prs` is never finished: it becomes free
only by release or by going stale after the TTL. A merger between batches holds such a lock, and
it must not be taken over mid-run.

**Lost-holder exit code.** Waiters rewrite the commit when they refresh, so a holder's CAS can
lose to a waiter even though the holder still holds the lock. `renew` and `release` therefore
re-read and retry until the write succeeds or the lock is no longer theirs, up to 10 attempts.
When the lock is no longer theirs (taken over), `acquire` as a renewal exits **4** (`lost`),
distinct from 1 (`busy`). `acquire --renew` states that the caller holds the lock, so a lock that
is no longer its own exits 4 even when it was taken over between two calls (T-2985). A merger
that gets 4 stops at once and merges nothing more. `release` exits 0 either way, as today.

**`release`** writes `state: free` and `freed: now`, and keeps the pruned queue, so the head waiter
takes the lock next. A session that releases and then acquires again is no longer in the queue,
so it joins at the tail. That fixes the #1859 → #1865 chain.

**`wait --pr N [--timeout 45]`** calls `acquire` every 30 s until it exits 0 (held) or 3 (PR
finished), or until the timeout (exit 1). It prints only changed messages. Agents run `make
merge-lock-wait PR=<n>` in the background and are woken when it exits, instead of polling.

**Mixed versions during rollout.** A session on an older checkout runs version-1
`merge_lock.py`. Such a client ignores waiter lines, can take a free lock that is reserved for the
head, and writes a commit without the queue. The waiters then re-register at their next refresh
(at most 3 min) and lose their place. That is a temporary loss of fairness, never of mutual
exclusion: the CAS is unchanged. The loss ends once every worker session starts from a `main`
that has T-2985, which takes one routine cycle of about 2 h.

**Write rate.** Each waiter refreshes every 3 min, plus acquires and releases. With 5 waiters that
is under 3 pushes a minute to a one-commit branch. Reads are one fetch of that branch per waiter
every 30 s.

### 2. The readiness handoff (T-2986)
A PR is **ready** when its head meets conditions (a) and (b) of the `steward` skill's merge policy:
every required CI job is green on the head, and every applicable reviewer approved that head. The
agent that established this marks it. Condition (c), "up to date with `main`", becomes the train's
job.

**Who may mark:**
- the PR's own worker, after its review panel approved;
- the steward, for an orphaned PR, after running the reviewers itself as decision 0050 requires;
- an opus run resuming at work-loop step 5c, after the full panel approved.

The merger never marks a PR and never reviews one.

**The marker.** `make mark-ready PR=<n> REVIEW="security=APPROVE architecture=APPROVE
quality=APPROVE [excel-fidelity=APPROVE]"` sets two things:
- the label `ready-to-merge` (only accounts with write access can set labels);
- a PR comment that ends with a machine-readable block:
  ```
  <!-- merge-train-ready v1
  head: <40-hex sha>
  prev: <40-hex sha> | none     # the previously marked head, when verdicts carry over
  task: T-XXXX | none           # none for task-only and steward PRs
  reviewers: security=APPROVE architecture=APPROVE quality=APPROVE excel-fidelity=n/a
  marked-by: <id>
  -->
  ```
  The block may be followed by exactly one copy of the attribution footer a session's GitHub
  write path appends (`\n\n---\n_Generated by [Claude Code](https://claude.ai/code)_`), itself
  optionally followed by one newline, and by nothing else (T-3164, T-3177, decision 0175).

`mark-ready` refuses (exit 1, with the reason) when:
- the PR is a draft or comes from a fork;
- the local `HEAD`, `refs/pull/<n>/head` and the PR's `head.sha` differ;
- a required job on that head is pending or failed;
- the task file on the head (the id comes from the branch name) is not `status: done` with an
  `## Outcome`;
- an applicable reviewer is missing from `REVIEW` or did not approve. Applicability is defined in
  one place, the `review` skill (`.claude/skills/review/SKILL.md`, step 3): excel-fidelity is
  required exactly when the diff touches `crates/rustsheet-engine`, `crates/rustsheet-xlsx`,
  `eval/suites` or `eval/baseline`. `merge_ready.py` holds the same list, and T-2986 adds a test
  that fails when the two differ. That test reads `.claude/`, so T-2986 adds the skill file to
  `CODE_READ_PATHS` (decision 0066).

It does not post a second marker for a head that already has one.

**Write fallback.** If T-3001 found that `gh api` writes fail in a session, T-2986 ships a
minimal fallback: `mark` prints its two writes (the label, and a comment whose body the script
generates in full) as step-mode JSON actions, using the schema of section 4 "GitHub access". The
session performs them, and `verify` confirms the result. T-2995's step mode is that same schema,
extended to the merger's actions.

**Carry-over.** A worker that merges `main` in after marking (to fix a conflict-free collision, or
when told to by an ejection) may carry the verdicts over only through `mark-ready --carry`. That
option records `prev:` and checks mechanically that the PR's own diff did not change:
- the new head is a merge commit whose first parent is `prev` and whose second parent is on
  `origin/main`;
- `git merge-tree --write-tree` of those two parents reproduces the new head's tree exactly,
  which means a clean merge with no edits.

Any other push needs the reviewers that the change concerns to run again, as today.

**Invalidation.** A marker names one head sha, so any push makes it stale without anyone doing
anything. The merger treats a marker whose `head` differs from the PR head as absent. It removes
the label and comments `readiness invalidated by a push (now <sha>); mark again after CI and
review`.

**Trust (owner decision, v1).**
- The verdicts in a marker are an **attestation, not a verification**. All agent sessions push as
  the owner's account, and today's flow already trusts the merging session's own review table
  (steward merge policy (b)). The marker keeps exactly that trust: it moves the attestation into
  a parseable comment.
- **Accepted residual risk, unchanged from today:** a buggy or prompt-injected worker session
  could mark a PR ready with fabricated verdicts, just as it could merge one today.
- The v2 follow-up, T-2997, replaces the attestation with per-reviewer verdict records bound to
  the head sha ("Verdict records (v2)" below).

**Verdict records (v2, T-2997; decision 0178).**
- **What they can and cannot prevent.** Every session, and every reviewer subagent inside one,
  acts as the owner's single GitHub account on the same machine, and the reviewers have no GitHub
  write tools, so the marking session posts whatever a reviewer returns. Nothing a reviewer can
  write is out of that session's reach, and a signing key would sit where the session can read
  it. So no record format stops a deliberately fabricating (buggy or prompt-injected) marking
  session; that residual risk stays as accepted in "Trust". Once `verdict_records = "require"`
  (none of this is enforced in `warn`), records do stop:
  - an **accidental stale panel**: verdicts given on an earlier head (before a fix push) and re-used for the
    new one. Each record names the head it reviewed, and V4 rejects one for any other head. The
    `head:` is the posting session's own claim, so this holds against a buggy session, not a lying
    one, and only because `make verdict-record` refuses when the local HEAD is not the PR head.
    Each reviewer's report starts with the `git rev-parse HEAD` it ran against, so a re-posted old
    report under a new head is visible in the audit;
  - a **verdict with no review behind it**: a `REVIEW=` string typed without the panel running
    finds no records, so `mark` refuses and V4 fails;
  - **silent fabrication**: each record carries the reviewer's own report, so a fabricated
    verdict is a visible comment the steward's spot-audit can check against the diff. It is
    auditable (GitHub marks an edited comment and keeps its history), not tamper-proof: the same
    account can edit or delete it, which is inside the residual risk.
- **Format.** One PR comment per reviewer, posted by the marking session right after that
  reviewer returns (`make verdict-record PR=<n> REVIEWER=<name> VERDICT=<v> REPORT=<file>`, direct
  or in step mode). The body is the reviewer's returned report and nothing else (no tool
  environment, no tokens), with every `<!--` and `-->` in it neutralised (a space inserted) so
  report text, which derives from untrusted PR content, can neither open a block nor close one;
  then the block. Truncation to GitHub's comment limit cuts the report only, with a marker line,
  and never the block. The block:
  ```
  <!-- merge-train-verdict v1
  head: <40-hex sha>       # the HEAD the reviewer read: the local HEAD, which must equal the PR head
  reviewer: security | architecture | quality | excel-fidelity
  verdict: APPROVE | REQUEST_CHANGES
  -->
  ```
  A diff digest was weighed and dropped: the head sha already fixes the tree, and with it the
  diff against the merge base, so a digest adds no binding the sha lacks.
- **Which records count.** Only records by a writer (V1's permission filter), and only the one
  block that ends the body (optionally followed by the decision 0175 attribution footer); a body
  with any other `merge-train-verdict` block in it is not a record. A record whose
  `updated_at` differs from its `created_at` (edited) is ignored, so an edit cannot turn a
  `REQUEST_CHANGES` into an `APPROVE`; a corrected verdict is a new record. "Newest" orders by the
  server's `created_at`, never a self-declared field: since edited records are dropped, the two
  server keys agree on every record that counts, and an edit fails closed (the edited record
  stops counting) instead of re-ordering anything. A record posted after the marker counts (a
  panel re-run on the same head), and the merger re-reads the records at every V4 check, so a
  deleted record or a newer `REQUEST_CHANGES` takes effect at the next merger run.
- **Checks.** `mark` refuses unless, for each applicable reviewer, the newest record by a writer
  (V1's permission filter) for the marked head says `APPROVE`; the marker's `reviewers:` field is
  then derived from the records, not typed. V4 becomes a verification: the same lookup by the
  merger, for the marker's `head`, or for its `prev` when the marker carries verdicts over (the
  merger re-runs the carry check of "Carry-over" on `prev` and `head`). A `REQUEST_CHANGES` record
  newer than the newest `APPROVE` for that head fails V4.
- **Rollout.** A `verdict_records = "warn" | "require"` key in `scripts/merge_train.toml`
  (default `warn`): in `warn` the merger reports a PR whose markers lack records in its run
  summary and still treats V4 as an attestation; the steward flips it to `require` (a decision
  record) once a week of marked PRs all carry records. The review skill posts the records in both
  modes. As built (T-3207): a missing key reads as `warn`, a switch file that does not parse or a
  value that is not a mode as `require` (fails closed); in `warn`, `mark` takes a missing
  reviewer's verdict from `REVIEW=` or the carried marker with a warning, and every V4 record gap
  (a missing or non-`APPROVE` newest record, or a carry-over that no longer holds) is a warning
  line in `verify` and the merger's run log, never an unlabel.
- **Out of scope.** The reviewers' criteria; a second GitHub identity or an external signer
  (owner steps, if the residual risk ever needs closing).

**Verification by the merger** (it does not review again). For each open PR with the label:
- **V1, author.** Read the PR's comments and keep only those whose author has `admin`, `maintain`
  or `write` permission on the repository (`GET /repos/{o}/{r}/collaborators/{user}/permission`;
  `author_association` alone is not trusted). Among those, the newest marker by `updated_at`
  counts. It is pinned by comment id and `updated_at`, and editing a comment makes it a new
  marker for FIFO purposes. Comments from anyone else are **ignored with no side effects**: the
  merger never comments, unlabels or ejects because of them. An outsider therefore cannot
  invalidate or delay a PR by posting a fake marker. If no qualifying marker exists, the PR is
  skipped silently.
- **V2, head.** `head` equals the PR's current `head.sha`, and fetching `refs/pull/<n>/head` gives
  that sha. Failure: remove the label and comment (invalidated, as above).
- **V3, CI.** The required jobs on that sha are green by section 4's "green" rule.
  - Pending: skip, and keep the label.
  - Red: remove the label and comment `CI red on <sha>: <jobs>`.
- **V4, verdicts.** The verdicts name every applicable reviewer, each with `APPROVE` (an
  attestation, see Trust; with verdict records in `require` mode, a verification, see "Verdict
  records (v2)"). Failure: remove the label and comment.
- **V5, PR state.** The PR is not a draft, its base is `main`, its head is in this repository, and
  its task file on the head is `status: done` with an Outcome (not `blocked`). Failure: remove
  the label and comment.
- **V6, reviews.** No review in state `CHANGES_REQUESTED` was submitted after the marker's
  `updated_at` by an author who passes V1's permission lookup (`admin`, `maintain` or `write`).
  Reviews by anyone else are ignored with no side effects, just as comments are. Failure: remove
  the label and comment.

V2–V5 read only repository-controlled data: the PR's `head.sha` and `refs/pull/<n>/head`, check
runs created by the GitHub Actions app, the PR's labels, draft flag, base and head repository,
and the task file in the head's tree. Nothing an outsider can write (comments, reviews, PR bodies)
feeds them. Only V1 and V6 read user-authored objects, and both filter by permission first.

The FIFO key is the marker's server-side `updated_at`, never a self-declared field.

**As built (T-2986, decision 0149).** In a session the permission read fails, and then V1 and V6
count `author_association` `OWNER` or `COLLABORATOR` on a user-owned repository only. The green
rule tells `run_heavy=false` by running the CI classifier (`ci_changes.classify_range`) on the
PR's base and head, because the job log is not readable from a session.

**As built (T-3040, decision 0161).** `make mark-ready` takes the verdicts from `REVIEW` or,
with `CARRY=1`, from the newest earlier writer marker; the two are exclusive. It writes the
comment before the label. `STEP=1` prints the two writes as step-mode JSON lines (`comment` with
the full body, `label` with the PR's whole label set afterwards); without it the script writes
through `gh_client` and runs `verify` on the result. `make merge-labels` creates the train labels.

**Comment hygiene.** The merger's comments quote no user text. Any `@` in names or reasons it
echoes (job names, PR titles) is escaped (`@` → `@​`), so a comment never notifies anyone.

### 3. Who runs the merger
**Primary: a new owner-created routine**, `rustsheet merger`. It runs on Sonnet every 30 minutes
(`5,35 * * * *`). Each run keeps working for a 40-minute window:
- every 2–3 min it lists PRs labelled `ready-to-merge`, and runs one batch when any is ready;
- its wait for the lock is capped at the time left in the 40-min window, so it acquires the lock
  no later than minute 40, and it starts no new batch after minute 40;
- `HOLD_BUDGET` (40 min) starts when the lock is acquired, and a hold never exceeds it
  (section 4, step 6).

**Real limits.** A hold is at most 40 min. A session is at most about 40 (window) + 40 (last
hold) + about 3 (start-up and release) ≈ 85 min. Consecutive runs overlap by about 10 min, so
coverage is close to continuous. Two overlapping runs coordinate only through the lock's FIFO
queue: while one holds the lock, the other waits as a waiter (`pr: train`).

The work is mechanical and scripted (`make merge-train`), so it needs no opus tier. Agents cannot
create routines, so the owner creates it (Owner steps). The prompt lives in routines.md (T-2996).
The routine also needs the fallback below, because it can be late, disabled or dying.

**Expected behaviour.** The source is the performance review's discrete-event simulation
(`sim2.py`, `sim3.py`, `sim5.py`, `sim6.py`), plus `sim7.py`, which reruns `sim6.py` with this
spec's final rule. None of these scripts is committed. The final rule is: `CI_TIMEOUT` = 10 min;
the per-wait n of section 4, step 6; the known-green prefix merged on a budget cut-off; and resume
always on.

Main rows use CI 5.4 min, 1.0 min of setup per batch and 0.6 min per PR merged. The values in
brackets are the 4.9-min variant (CI 4.9, setup 0.5, 0.5 per PR).

| arrivals | p | marker → merge p50 / p90 | mean batch | lock busy | merge-stage CI runs per merged PR |
|---|---|---|---|---|---|
| 3.8/h | 0 | 8.6 / 11.4 min (7.4 / 9.7) | 1.18 | 38% (33%) | 0.85 (0.87) |
| 3.8/h | 0.05 | — | — | — | ≈ 0.95 (performance review) |
| 3.8/h | 0.09 | 8.7 / 12.7 min (7.5 / 10.8) | 1.22 | 42% (36%) | 1.08 (1.10) |
| 7/h | 0 | 8.9 / 12.2 min (7.7 / 10.4) | 1.41 | 60% (54%) | 0.71 (0.76) |
| 7/h | 0.09 | 9.4 / 23.5 min (8.0 / 16.8) | 1.57 | 67% (60%) | 0.93 (0.97) |
| 10/h | 0.09 | 11.7 / 42 min (9.2 / 27) | 2.16 | 84% (77%) | 0.81 (0.87) |

Across the review's other variants: lock busy 34–45% at 3.8/h and 56–71% at 7/h; at 7/h, p90 is
11–26 min; at 10/h, p90 is 27–64 min (`sim2.py` plain 43, the earlier rule with resume 38, the
earlier rule without resume 64). The confirmation run costs about 0.09 runs per merged PR at
p = 0.09, and it buys the flake check of section 6. With the final rule, cut-offs are rare
(0.03 a day at 3.8/h, 0.35 a day at 7/h, p = 0.09). The simulations charge the push to CI
time: setup is one fixed cost per hold. A rerun with the confirmation cut-off of section 6
(`sim7r.py`) at CI 5.4, 9 and 10 min found no livelock: 0.98–1.05 runs per merged PR at 3.8/h and
0.75–0.92 at 7/h (p = 0.09); near the timeout (CI 9–10, seeds 1–3), p90 rises to 25–35 min at 3.8/h and 62–110 min
at 7/h, since each hold then fits fewer runs.

The knee lies between 7/h and 10/h. **Revisit** (pipelined trains, a larger effective batch, a
shorter CI) when arrivals stay at about 8–10/h, or when p90 latency in the "over 6/h" bucket
passes 30 min (Metrics).

**Stall fallback** (`train` mode only). Any session may run one `make merge-train` pass when all
of these hold:
- a PR has been ready for more than 45 min (by the marker's `updated_at`);
- the last CI run on `merge-train/current` was created more than 45 min ago (the workflow-runs
  API's `created_at`, a server time, not a commit date);
- the lock queue holds no `train` waiter and the lock is not held by a `train`.

"One fallback at a time" is a check-then-act, so two sessions can both pass the check. That is
harmless: the lock's FIFO queue serialises them, and the later pass finds nothing ready and exits.
The fallback replaces today's "wait for the lock and merge it yourself" as the safety net.

**The steward** ends its daily run with one pass, after marking orphaned PRs ready.

**Lock ownership.** The merger holds the ordinary merge lock with `pr: train`. `prs` lists the
batch, or is empty between batches. The merger waits for the lock with `merge_lock.py wait`, like
any waiter. In `train` mode only mergers take the lock, plus anyone merging the emergency
`lock`-flip PR (section "The switch").

### 4. The batch algorithm (T-2987, T-2994, T-2995; red trains T-2998)
Constants:
- `BATCH_MAX = 5`;
- `TRAIN_REF = refs/heads/merge-train/current` (`merge-train/shadow` in shadow mode);
- `CI_TIMEOUT = 10 min`: the longest a CI wait may take. PR and train CI runs take about 4.9–5.4
  min wall, so 10 min is about twice a normal run. At 15 min, the budget gate below cut the
  section 6 example (culprit at `P_5` of 5) off at its fourth run. Without a resume that
  livelocked: the performance review's `sim6.py` measured 25–237 runs per merge. A run that
  really takes longer than 10 min is an infrastructure failure (section 6);
- `MERGE_BUDGET = 2.5 min` per PR that a CI run can merge: the pre-squash wait of up to 2 min,
  the merge call, and the checks. Its consequence: a full batch of 5 reserves 12.5 min after its
  CI wait, so a 5-PR train needs `10 + 12.5 = 22.5` min of hold budget left before its CI wait
  starts;
- `HOLD_BUDGET = 40 min`, counted from the moment the lock is acquired.

**Write policy** (one rule, applied by the script's single GitHub client):

| mode | lock | pushes | merges | labels and comments |
|---|---|---|---|---|
| dry run (`DRY_RUN=1`, any mode) | none | none | none | none; would-be unlabels and ejections are printed |
| `shadow` | none | `merge-train/shadow` only | none | only the housekeeping: `ready-to-merge` removed from closed PRs. Would-be unlabels and ejections go into the shadow commit's predicted outcome |
| `train` | yes | `merge-train/current`, `merge-train/revert-*` (a revert PR's branch, step 7), `merge-train/tasks-*` (a task-only PR's branch: the P0 of step 7, the P1 of section 6) | yes, the batch's PRs only; a revert PR only once review has marked it ready, like any PR | everything in this section, and opening the revert and task-only PRs |
| `lock` | — | — | — | the merger stops at once. Only the steward does housekeeping (section 8) |

The script installs an exit handler (on success, error or signal) that releases the lock if it
holds it.

**Stop checkpoints** (`train` mode). At each checkpoint the script does two things:
- it lists open PRs labelled `merge-train-stop` and honours only one that currently passes `make
  merge-flip-check` (see "The switch"). The label is removed, with a comment, from any other PR;
- it runs `git ls-remote origin refs/heads/main`, and only when `main`'s sha has changed does it
  fetch and re-read `origin/main:scripts/merge_train.toml` (failing closed).

If the mode is no longer `train`, or an honoured stop label is present, the script stops at once
with nothing more merged, releases the lock and reports. A labelled PR whose check cannot run git
is honoured too (stopping is always safe), and is not unlabelled. `make merge-mode` prints
`train (stop pending: #n)` while a train would honour PR n's label; it only reads, so it never
removes a label (T-3055). The checkpoints are:
- **right after lock acquisition**, before the build and push, so that a pending flip does not
  cause build–push–release loops;
- before the CI wait;
- on every CI poll (every 60 s);
- before each squash.

During a hold, a flip acts through its stop label, because the flip PR cannot merge while the
train holds the lock. The mode re-read matters between holds, and after a flip that merged. A
flip therefore takes effect within about 1 min of its stop label, or of its merge.

1. **Lock.** In `train` mode, run `merge_lock.py wait --train <run id>`, with the timeout capped at
   the time left in the run's 40-min window. On exit 1, stop the run. Shadow mode takes no lock.
2. **Collect.** List open PRs labelled `ready-to-merge` and run V1–V6 on each. Section 2 gives the
   side effects of each failure, and they apply in `train` mode only; the other modes report
   them. A PR whose `mergeable_state` is `dirty` is ejected with the reason `conflict with main`.
   Sort the rest by marker time, oldest first, and take up to `BATCH_MAX`. If none is left,
   release and report "no ready PR". Housekeeping in `train` and `shadow` mode: remove
   `ready-to-merge` from closed PRs, because in `shadow` mode a worker merges ready-labelled PRs by
   the per-PR flow.
3. **Build.** Let `M0` be the sha of `origin/main`, freshly fetched. Create two scratch worktrees
   outside the session's checkout, one for the train at `M0` and one, read-only, for `M0`'s
   scripts. For each PR k in order:
   - fetch `refs/pull/<k>/head` and require the fetched sha to equal both `head.sha` and the
     marker's `H_k`. Branch names are not used, because they can be re-pointed;
   - **independence check.** For every PR j already in the train, every commit printed by
     `git merge-base --all H_j H_k` must be an ancestor of `M0`. That covers the criss-cross
     case, where there are several merge bases. A PR stacked on another member of the batch, or
     two PRs that merged the same non-`main` commit, is **deferred**: it stays ready and joins a
     later batch that no longer holds the other PR. Section 5 explains why;
   - `git merge --no-ff --no-edit H_k` (merge-ort; never a rebase or an edit of a PR branch). On
     a conflict, abort the merge. If `H_k` merges cleanly onto `M0` alone, it conflicts only with
     an earlier member: defer it. Otherwise eject it (`conflict with main`).
   - `T_k` is the new merge commit.
4. **Id checks.** Run `python3 -I <M0 scripts worktree>/scripts/tasks.py validate --root <train
   worktree>` and the same for `check_decisions.py`, with the working directory outside the train
   tree. That is decision 0066's rule: a PR cannot change how its own tree is judged, and `-I`
   with an outside working directory means no module from the train tree is imported. T-2987 adds
   `--root`. On a duplicate task id or decision number, eject the PR that adds the later copy in
   train order, or the PR itself when the duplicate is with `M0`. The reason is `id collision
   with <id> on main / in #j; renumber per decision 0121 (2)`. Then rebuild from step 3, at most
   `BATCH_MAX` times.
5. **Push.** Push `T_n` to `TRAIN_REF` with `--force-with-lease=<sha read at step 3>`. In shadow
   mode a lease failure means that another shadow build won the race: skip this round. In
   `train` mode it cannot happen under the lock, so it is an infrastructure failure. The commit
   message lists the batch (PR numbers and head shas), any known-red prefix (section 6), and in
   shadow mode the predicted outcome. Every train is therefore a durable record that the commits
   API can read by sha. In `train` mode, renew the lock with `prs` = the batch; renewal exit 4
   means stop.
6. **CI.** Start a wait only if at least `CI_TIMEOUT + n × MERGE_BUDGET` remains of
   `HOLD_BUDGET`. Here **n is the number of PRs that this run can merge**:
   - for the full train, the batch size;
   - for a bisection step testing the prefix `P_1..P_mid`, `mid`;
   - for the confirmation run, 1 (section 6).

   If the gate fails, first merge any known-green prefix (section 6, "Budget cut-off"), then
   release, and the remaining PRs keep their readiness. A hold therefore never exceeds
   `HOLD_BUDGET` (40 min), every hold that ran a green CI makes progress, and a hold that ran
   only red CIs records them, so no later hold repeats them. Wait for the CI run
   that matches **all** of these:
   - workflow file `.github/workflows/ci.yml`;
   - event `push`, or `workflow_dispatch` for the one dispatch fallback below;
   - `head_branch` = the train branch;
   - `head_sha` = `T_n`, the sha this run built and pushed. Anything else on the branch is ignored.

   Poll every 60 s (a stop checkpoint), and renew the lock every 5 min. If no run appears within
   3 min, dispatch the workflow on the branch once; it is classified like the push (section 9). If
   none appears after that, or the run exceeds `CI_TIMEOUT`, stop with an infrastructure failure.

   **Green**, counting only check runs created by the GitHub Actions app (app id 15368):
   - `changes (docs-only?)`, `hygiene (...)` and `rust (fmt, clippy, test)` (the summary job)
     each concluded `success`;
   - `eval (...)` and `python (...)` concluded `success`, or `skipped` only when `changes`
     concluded `success` with `run_heavy=false` (read from its job log line `code=false`);
   - the run itself concluded `success`.

   A `cancelled`, `timed_out` or superseded run is **never** green. The same rule judges V3 on PR
   heads (event `pull_request`).
7. **Green: merge in order** (`train` mode only). Renew the lock first; exit 4 means stop with
   nothing merged. Check that `origin/main` is still `M0`. If not, someone merged without the
   lock: release, report, file a P0 and stop. No merge lands on an untested tree. Then, for k = 1
   to n:
   1. **Stop checkpoint**, then **renew** the lock with `prs` = the PRs not yet merged. Exit 4
      means stop.
   2. **Pre-squash check (mandatory).** Poll `GET /pulls/<k>` every 5 s, for at most 2 min, until
      `mergeable` is non-null and `merge_commit_sha` is a commit whose first parent is the current
      `main` (`S_{k-1}`, or `M0` for k = 1). GitHub recomputes the test merge after every push to
      `main`. Read that commit (`GET /git/commits/<merge_commit_sha>`) and require its `tree.sha`
      to equal `tree(T_k)`: git object ids are the same on both sides, so no fetch is needed. A
      timeout or a mismatch stops the sequence at k.
   3. **Squash-merge** PR k with `expectedHeadSha = H_k`. The commit text is described under
      "Squash commit text".
   4. **Post-squash check.** Fetch `origin/main` and require that its first parent is the
      previous `main` and that its tree equals `tree(T_k)`.
   5. Remove the label and comment `merged by merge train <T_n> (batch #a, #b, ...)`. Try to
      delete the PR branch; a 403 from the proxy is ignored (T-1938).

   **Response to a mismatch.**
   - **Before the squash** (the pre-squash parent or tree differs, or `origin/main` moved): nothing
     of ours landed. Stop, file a P0 (`merge train: main differs from the tested train at #k`),
     release and report. A foreign commit that moved `main` is not ours to revert. The P0 handler
     judges it.
   - **After the squash** (`S_k`'s parent or tree differs): our `S_k` is on `main` with an
     untested tree. The merger stops merging and **prepares the revert**; it never merges it:
     1. on a scratch worktree at the current `origin/main`, run `git revert --no-edit S_k` and
        push the result to `merge-train/revert-<S_k sha>`;
     2. open a PR from that branch titled `revert: merge train mismatch at #k`. Its body is
        generated by the script and names `S_k`, the expected tree and the actual tree;
     3. file the P0 as a task file in a separate task-only PR, and name the revert PR in it;
     4. release the lock and report.

     The check reads `S_k` from the merge reply (or the PR's `merge_commit_sha` when the reply is
     lost). When `S_k` itself has the tested parent and tree but `main` has a foreign commit on top
     of it, that is the "before the squash" case for the next PR: a P0, no revert (T-3055).

     **The revert PR is reviewed and merged like any other PR, with P0 priority, under the normal
     flow of the current mode.** The P0 handler (any agent) runs the review panel, then merges it
     under the lock (mode `lock`) or marks it ready (mode `train`), where it rides the next batch
     as an ordinary PR. No review waiver applies to reverts. Revert PRs get `pull_request` CI only,
     because the push trigger matches only `merge-train/current` and `merge-train/shadow`
     (section 9).

   **Task filing** (T-3055). The P0, and the P1 of a red train that did not reproduce (section 6),
   are task files written with `main`'s `tasks.py new` (milestone M2, lane `harness`, tier `opus`,
   so the id is collision-free) in a scratch worktree at the current `main`. Their text is the
   script's own (shas, PR numbers, run links). One commit on `merge-train/tasks-<first id>` and
   one task-only PR per hold carry them. A failure to file is logged with the unfiled tasks.

   A merge call that fails (the head moved, so `expectedHeadSha` refused) also stops the sequence
   at k, without a revert, because nothing landed. PRs after k are not merged in this pass. The
   squashes that landed equal **a prefix of the tested train**, and merging past the gap would
   not.
8. **Red: bisect.** See section 6 (T-2998).
9. **Release** the lock. While the run's window is open, go back to polling (section 3). The
   merger re-enters the FIFO queue at the tail, like anyone else.

**Squash commit text.**
- The title is the PR title, with control characters stripped and truncated to 100 characters.
  In it, `@` and `#` are each followed by a zero-width joiner, which neutralises mentions and `#n`
  references. A GitHub closing keyword (`close`, `closes`, `closed`, `fix`, `fixes`, `fixed`,
  `resolve`, `resolves`, `resolved`) is neutralised the same way, so no issue is closed or linked
  by accident. Then the script appends its own ` (#k)`.
- The body holds only trailers, parsed from structured data:
  - the `Co-Authored-By:` lines of the PR's commits (`GET /pulls/<k>/commits`), kept only when
    they match `Co-Authored-By: <name> <<email>>` with a name free of `@` and `#`, and
    de-duplicated;
  - the PR's `Claude-Session:` trailers, kept only when they match the session URL pattern.
- Free text from PR bodies or comments never enters the commit.

**Dry run.** `make merge-train DRY_RUN=1` runs steps 2–4 and prints the batch, the deferrals and
the would-be ejections and unlabels. It makes no write of any kind (the write policy above).

**Shadow** (T-2995; `make merge-train SHADOW=1`, used when the mode is `shadow`) runs steps 2–6
and the bisection of section 6 against `M0`. It follows the write policy above, and in addition:
- it pushes a new shadow train only when the ready set or `main` changed since the last one, so
  CI is not spent on repeats;
- **the predicted outcome** goes into each shadow train's commit message (it is updated by a
  follow-up commit when the bisection runs). For each PR it records `merge`, `defer`, or
  `eject:<reason>` (`ci-red`, `conflict`, `id-collision`, `invalidated`), together with the
  would-be unlabels;
- **the shadow tree check.** For the first PR of each shadow train, whose GitHub test merge is on
  `M0` while `main` has not moved, it compares the `tree.sha` of that PR's `merge_commit_sha`
  with `tree(T_1)`, and records `tree-check: match | mismatch | n/a` (n/a when `main` moved in
  the meantime). This is the only part of step 7 that shadow mode can exercise;
- a shadow run that was cancelled or superseded is recorded as `no result`, never as green.

**When a shadow prediction agrees** (computed by `make merge-train-stats`, T-2995). It compares
each PR's prediction with what happened to that PR within 24 h, read from the PR's later state,
its later head commits and their check runs:
- `merge` agrees if the PR later merged by the per-PR flow with the same head sha, or with a head
  that is only a clean merge of `main` onto it (the `--carry` rule), and its under-lock CI run was
  green. It disagrees if that run was red, or if the PR needed a non-merge change before merging.
- `eject:ci-red` agrees if the PR's head later went red under the lock, or the PR received a
  non-merge change before it merged. It disagrees if the PR merged unchanged with green CI.
- `eject:conflict` and `eject:id-collision` agree if the PR's next head merged `main` in with a
  conflict resolution, or renumbered a file, before it merged.
- `defer` and `no result` are not scored.

**GitHub access.** One client, `scripts/gh_client.py`, with a fake for tests, owned by T-3001.
`merge_ready.py`, `merge_train.py` and `merge_lock.py`'s PR-finished read all use it: T-3001 moves
that read off `merge_lock.py`'s own urllib code. The production implementation calls `gh api`.
Reads were verified through the session proxy on 2026-10-28. T-3001 probes the writes (merge,
label, comment, branch push of `merge-train/*`, and `GET /git/commits` on a test merge) from a
routine session, **before** any slice depends on them, and records the answer here.

**T-3001's probe** (2026-10-28, an opus worker run, through `scripts/gh_client.py`):
- **Reads work** through the session proxy: a PR, the open PRs (paged with `page=`), a PR's
  comments, a head's check runs and workflow runs, and `GET /git/commits/<sha>` on a PR's
  `merge_commit_sha` (#2160's test merge `fae51abf`, parents `cb57b875` and `14419263`).
- **`GET /collaborators/{user}/permission` is refused** by the proxy (HTTP 403, "Access to this
  GitHub API path is not permitted through this proxy"). So in a session, T-2986's V1 and V6
  cannot read an author's permission. The candidate source is each comment's and review's
  `author_association` (present on #1718's comments, `OWNER`). On this user-owned repository,
  `OWNER` and `COLLABORATOR` mean write access. `MEMBER` would need the permission API after an
  organization move. T-2986 decides and tests it. `permission()` stays in the client for CI,
  where the token can read it.
- **A branch push works**: `git push` of an empty-tree commit to `merge-train/probe` (`62eebcac`)
  succeeded, and ci.yml starts nothing for it (decision 0147). The session proxy refuses branch
  deletion (T-1938), so the branch stays; later probes overwrite it.
- **`gh api` writes did not run**: the session's tool-permission layer refused the comment, label
  and unlabel calls (it judged them unrequested writes to a connected app) before they reached
  GitHub. A routine session therefore cannot count on `gh api` writes. The GitHub MCP tools did
  write in the same run (opening #2160 and its squash merge under the merge lock, with a wrong and
  then the right `expectedHeadSha`; see T-3001's Outcome), so the **step mode** below is the write
  path for sessions; `gh_client` writes serve CI (`GITHUB_TOKEN`) and an owner-approved session.

If writes fail, T-2995 adds a constrained **step mode** (T-2986's minimal fallback uses the same
schema; the "As built" note below supersedes the enum and the re-run of these bullets):
- the script prints the next write as one JSON action. The action type comes from a closed enum
  (`merge`, `label`, `unlabel`, `comment`), and the script fills every field: PR numbers validated
  as integers that the script itself selected, label names from a fixed list (`ready-to-merge`,
  `merge-train-ejected`), comment bodies generated by the script in full, and `expectedHeadSha`
  for merges;
- the session performs exactly that action with the GitHub MCP tools and runs the script again;
- the script reads GitHub's state back and aborts on any difference from the expected result;
- the session never adds, skips, reorders or changes an action.

**As built (T-3077, decision 0169).** `make merge-train TRAIN=1 STEP=1` (or `SHADOW=1 STEP=1`) runs the
merger with `scripts/merge_step.py`'s `StepClient`: reads go through `gh_client`, and the eight writes
(`comment`, `label`, `unlabel`, `merge`, `create_pull`, `delete_branch`, `dispatch`, and `set_base`,
which re-sets a PR's base to `main` so GitHub recomputes a stale test merge, decision 0200) are actions validated
against one schema, which `make mark-ready STEP=1` also uses. The run stays alive and keeps the lock:
it prints each action as one JSON line (with a `seq`), the session performs it and runs `make
merge-train-ack SEQ=<n>` (`REFUSED=1` when GitHub or the tool refused it), and the script reads the
state back. Done must read exactly as expected, refused must read unchanged (and then fails like a
refused direct write); anything else, no ack within 10 min, or an ack for another `seq` aborts the
run (exit 2, lock released). Comment and PR bodies read back with or without the session's footer.

### 5. Why `main` ends with the tested tree, and how a difference is caught
Notation:
- `M0` is `main` at step 3, `H_1..H_n` are the PR heads, `T_0 = M0`, and
  `T_k = merge(T_{k-1}, H_k)`;
- `S_k` is the k-th squash commit on `main`;
- `A(x)` is the set of ancestors of commit `x`, `x` included.

CI tested `tree(T_n)`. Intermediate prefixes are discussed at the end of this section.

**The argument, which holds under the lock discipline:**
- *Each squash is the same three-way merge as the train step.* A squash merge of PR k creates a
  commit on `S_{k-1}` (or on `M0`). Its tree is the three-way merge of `H_k` into that commit,
  with the bases `merge-base --all (S_{k-1}, H_k)`. The train step merged the same `H_k` into
  `T_{k-1}`, with the bases `merge-base --all (T_{k-1}, H_k)`.
- *The inputs are equal, by induction.* `tree(S_0) = tree(T_0)`. Suppose
  `tree(S_{k-1}) = tree(T_{k-1})`. The "theirs" side is the same commit `H_k`. The common
  ancestors with `H_k` are:
  - on the `main` side, `A(M0) ∩ A(H_k)`, because no PR head contains a squash commit;
  - on the train side, `(A(M0) ∩ A(H_k)) ∪ ⋃_j (A(H_j) ∩ A(H_k))`.

  The independence check (every merge base of `H_j` and `H_k` is in `A(M0)`) makes each
  `A(H_j) ∩ A(H_k)` a subset of `A(M0)`. So both sides have the same common ancestors, hence the
  same merge bases. Equal inputs give `tree(S_k) = tree(T_k)`.
- *Conflict-free by construction.* Step 3 kept only heads that merged cleanly.

**The guarantee is detective, not preventive.** GitHub's merge API guards only the head
(`expectedHeadSha`), not the base. Two things could break the argument:
- something moves `main` between two squashes, such as a human, a session that ignores the lock,
  or a stale lock that was taken over;
- GitHub's merge code resolves the same inputs differently from local git (rename detection, for
  example).

Neither can be prevented from here, so step 7 detects both:
- the pre-squash check compares GitHub's own test merge, recomputed on the current `main`, with
  `tree(T_k)`, and requires its first parent to be the `main` the merger expects;
- the post-squash check compares the result.

Any mismatch stops the train and files a P0. For a mismatching squash of ours that landed, the
merger also opens a revert PR (step 7). That PR is reviewed and merged under the normal flow,
with P0 priority. The remaining gap is a move of `main` *between* the pre-squash check and the
squash call, a window of seconds. The post-squash check catches it, and the revert PR removes the
untested result once it is merged.

**Intermediate states (decided: accepted, not tested).**
- `main` passes through `tree(T_1)` to `tree(T_{n-1})` for seconds each. `ci.yml`'s
  `concurrency: ci-${{ github.ref }}` with `cancel-in-progress` also applies to `main`, so the
  push runs of `S_1` to `S_{n-1}` are cancelled, and only `S_n` runs. `S_n`'s tree is the tested
  train tree.
- **Back-to-back trains.** The next train's first squash can land while `S_n`'s push run is still
  going, and it cancels that run too. `S_n` then gets no push run of its own, so in a busy period
  only the last squash has one, and with it `main`'s full eval report (decision 0054). This is
  accepted: `S_n`'s tree is the tested train tree, and the newest squash's run tests a tree that
  contains it. The daily steward run keeps one full report per day.
- Exempting `main` from `cancel-in-progress` would test every prefix, but it multiplies `main`'s
  CI cost by n, and those runs would only confirm prefixes of a tree that already passed. This
  spec accepts the untested intermediates, and the 0028 amendment says so.
- **A red `S_n`** on a tree equal to a green train means a nondeterministic test or a changed
  environment, not a bad merge. It is handled under "`main` red after a merge": re-run once. If it
  is red again, revert the whole batch (squash commits newest first) with a revert PR that is
  reviewed and merged under the normal flow, and file a P0. No prefix commit counts as known-good.
- **`git bisect` over `main`** may land on an untested prefix commit. Each prefix is the merge of
  PRs that each passed CI on their own head and as part of a passing train, so it usually builds.
  When one does not, `git bisect skip` is the right answer, not a bug report.
- A crash between squashes can leave `main` at an untested prefix until the next train. The next
  push run tests that prefix (it is the newest push), and the next train tests on top of it.

### 6. A red train: binary search for the first red prefix (T-2998)
For a red train `[P_1..P_n]` on `M0`, let `lo = 0` (the empty prefix, which is `M0`, known green)
and `hi = n` (known red). While `hi − lo > 1`:
- test the prefix `P_1..P_mid` on `M0`, with `mid = ⌊(lo + hi) / 2⌋`;
- if it is green, `lo = mid`; if red, `hi = mid`.

That takes ⌈log₂ n⌉ runs at most. Then:
1. **Merge `P_1..P_lo`** (step 7) when `lo > 0`. This tested prefix is exactly the tree that lands.
2. **Confirm the culprit `P_{lo+1}`** with at most one run: the train `[P_{lo+1}]` on the new
   `main`, whose tree equals the known-red prefix `lo + 1`.
   - Red: **eject** it. Remove `ready-to-merge`, add `merge-train-ejected`, and comment with the
     failing jobs, the run links and `<!-- merge-train-ejected v1 head: <sha> reason: ci-red -->`.
     Whether the PR broke alone or only in combination with the PRs before it, it is the PR that
     would merge later, so it is the one that fixes the clash (decision 0028).
   - Green: the failure did not reproduce. Do not eject. File a P1 task (`possible flaky test or
     interaction`) with both run links, since no test is called flaky without proof (steward
     skill). Then continue as if the prefix had been green.
3. **Re-test the rest** `P_{lo+2}..P_n` as one new train on the new `main` (step 3 onward).

For n = 1 the bisection is empty, and the confirmation is a re-run of the same tree. The
confirmation run is made on every red train, unconditionally. V3 already requires every job to
be green on the PR's own head, so a condition of the form "the failing job passed on the head"
would always be true. The run is the flake check, and the cost bound and the simulation both
count it.

**Cost bound per red train:** at most 1 (the full train) + ⌈log₂ n⌉ + 1 (confirmation) runs,
which is 5 for n = 5, followed by one train for the rest. The bound holds across holds, because
a cut-off resumes rather than restarting (below).

**Example: the culprit is `P_5` of 5**, with the step 6 gate (n = the PRs a run can merge):
- **With typical CI** (5.4 min, 1 min to build and push, 0.6 min per merge):

  | event | ends at | gate check before the wait |
  |---|---|---|
  | full train, red | 6.4 min | 1 + 10 + 12.5 = 23.5 ≤ 40 |
  | `mid = 2`, green | 11.8 min | 6.4 + 10 + 5 = 21.4 ≤ 40 |
  | `mid = 3`, green | 17.2 min | 11.8 + 10 + 7.5 = 29.3 ≤ 40 |
  | `mid = 4`, green | 22.6 min | 17.2 + 10 + 10 = 37.2 ≤ 40 |
  | merge `P_1..P_4` | 25.0 min | — |
  | confirm `P_5`, red; eject | 30.4 min | 25.0 + 10 + 2.5 = 37.5 ≤ 40 |

  That is 5 runs in **one hold of about 31 min**, with four PRs merged.
- **With every CI run at the 10-min timeout:**
  - full train red at 11 min; `mid = 2` green at 21 min; `mid = 3` green at 31 min;
  - `mid = 4` fails the gate (31 + 10 + 10 = 51 > 40);
  - so the merger merges the known-green `P_1..P_3`, records the state and releases (hold
    ≤ 40 min);
  - the next hold resumes on the new `main` with `[P_4, P_5]` known red: `mid = 1` (`P_4`) green,
    merge it, confirm `P_5` red, eject it.

  That is still 5 runs, now over **two holds**, with no third rebuild.

**Infrastructure failures** (a cancelled or timed-out job, a runner that never started, no run for
the sha) are not a PR's fault. Retry the same train once. If it fails again, release and stop the
run without ejecting anyone.

**Budget cut-off.** No CI wait starts unless `CI_TIMEOUT + n × MERGE_BUDGET` remains of
`HOLD_BUDGET`, where n is the number of PRs that run can merge (step 6). So a hold never exceeds
40 min. When the gate fails during a bisection **or before the confirmation run**, the merger:
1. merges the known-green prefix `P_1..P_lo` (when `lo > 0`; the reservation made for the earlier
   step that proved it green covers these merges);
2. records the state (before the confirmation, `known-red` is the single PR `P_{lo+1}`, so
   `hi − lo = 1` and the resumed hold goes straight to the confirmation run);
3. releases.

Every hold that ran a green CI therefore makes progress, and a hold that ran only red CIs is
never repeated. Without the confirmation case the train livelocks near the timeout: with the
culprit at `P_1` of 5 and every run at 10 min, the full train is red at 11 min, `mid = 2` red at
21, `mid = 1` red at 31, and the confirmation gate is 31 + 10 + 2.5 = 43.5 > 40. Restarting
would rebuild the same red train in every hold (the performance review measured tens to thousands of
runs per merge at CI 9–10 min, 39–4355 across rates and seeds). With the cut-off recorded, the next hold confirms `P_1` red, ejects it
and re-tests the rest: 4 runs over two holds. Fairness wins over finishing a
bisection, and the PRs not yet resolved keep their readiness.

**Resume is required, and it is the normal path.** The state is written into a commit on
`merge-train/current`:
- `bisect-base: <sha of main after the cut-off merges>`;
- `known-red: P_{lo+1}..P_hi`, with their head shas (this train, on that base, is known red);
- `known-green-prefix: 0`.

The next pass reads the last `merge-train/current` commit through the commits API. It resumes
the search (`lo = 0`, `hi = hi − lo`) without re-running the known-red train when two things
hold: `main` is still `bisect-base`, and those PRs' heads are unchanged. In `train` mode only the
merger moves `main`, so this holds whenever no flip PR or revert PR merged in between, which
makes resume the normal path rather than the exception. When it does not hold, the pass starts
again, at a cost of at most ⌈log₂ n⌉ + 1 extra runs, and the state record says why.

**After an ejection.** The ejected PR's worker is woken by the comment, through its PR activity
subscription. It fixes the PR per the `steward` skill and marks it ready again. If that session
has ended, the PR is resumed like any idle agent branch (worker prompt rules (1) and (2): an open
`agent/<lane>/*` branch idle for 2 h). The label `merge-train-ejected` tells the resuming run why
the PR is open, and is removed when the PR is marked again.

### 7. Crash, timeout and idempotency
The merger keeps no state between runs: every step is derived from `origin` and GitHub.
- **The merger dies while holding the lock.** The exit handler releases the lock when it can. If
  the process is killed, the lock goes stale 30 min after the last renewal (every 5 min), and the
  FIFO head takes it over (0075's TTL is unchanged).
- **Stale train branches.** `merge-train/current` and `merge-train/shadow` are overwritten by the
  next train (`--force-with-lease`). Their old CI runs are cancelled by the concurrency group, and
  a cancelled run never counts as green. The branches are never deleted, because the proxy
  refuses branch deletion (T-1938). Fixed names keep `tasks.py`'s branch scan from growing.
  Branches outside `agent/*` claim nothing (`AGENT_BRANCH`), and the ids on a train branch are a
  subset of its PRs' own.
- **A partly merged batch.** Merged PRs are closed, so they are no longer candidates. The rest
  still carry a marker on an unchanged head, so the next batch collects them first (oldest marker
  time) and tests them on the new `main`.
- **A merge call whose response was lost.** Read the PR state before retrying. A merged PR counts
  as merged, and its post-squash check runs.
- **The session budget.** See section 3: at most about 85 min, and the lock is always released
  before the session stops.

### 8. Interactions
- **Conflicts (decision 0020).** A PR that conflicts with `main` is ejected with `conflict with
  main`. Its worker resolves it at once, per 0020, and marks it again. A PR that conflicts only
  with an earlier member of its batch is deferred. In the next round it is either clean (the
  other PR was ejected) or conflicts with `main`, and is then ejected. Decision 0121 is
  unchanged. In `train` mode no worker merges `main` in just to be current.
- **Id collisions (0016, 0028, 0121).** Found before CI (step 4). The later PR is **ejected** and
  renumbers its own file per 0121 (2). The merger does not renumber:
  - a renumber is a push to another session's branch after review;
  - it can race a live session;
  - it is not mechanical (`depends_on` lists, links and decision citations change too).

  Open question 3 asks the owner about this.
- **Docs-only and task-only PRs.** Eligible like any other PR. Task-only PRs are exactly the ones
  that collide on ids (0075). Their marker says `task: none`, and V5's task-file check is skipped.
  A train is classified as a whole (section 9), so a train of docs-only PRs runs only `changes`,
  `hygiene` and the rust summary job.
- **The steward's PR** (`agent/steward/<date>`) is marked ready by the steward and merged by the
  train. The steward then runs its final merger pass (section 3).
- **Orphaned PRs (0050).**
  - In `train` mode, the steward's conditions for merging an orphaned PR become its conditions for
    marking one ready: the age and idle clocks, not a draft, not blocked, CI green, its own review
    panel, and the task done with an Outcome. The 0028 up-to-date condition drops out, because
    the train provides it. A PR its worker already marked ready is not orphaned.
  - In `lock` mode (including after an emergency flip), the steward may at once merge a PR
    labelled `ready-to-merge` per the per-PR flow, or remove the label, without waiting for the
    24 h orphan clock. The marker already records what that clock stood in for: finished,
    reviewed work. In `lock` mode the merger stops at once and does no housekeeping. The steward
    is the only one that removes `ready-to-merge` from closed PRs then.
- **Duplicates (0050).** The train merges the first of two PRs that implement one task id. The
  second then collides or conflicts and is ejected, and the steward closes it per 0050.
- **Required checks and auto-merge.**
  - When the owner adds the required checks to the `main` ruleset, the squash calls work
    unchanged, because those checks are green on every PR head (V3). The checks are `changes
    (docs-only?)`, `hygiene (...)`, `rust (fmt, clippy, test)`, `eval (Excel-compatibility gate)`
    and `python (tools + bindings)`.
  - **"Require branches to be up to date before merging" must stay off.** With it on, GitHub
    refuses PRs 2..n of a batch, whose heads do not contain the squash commits before them. The
    train is what keeps merges up to date, so decision 0028's earlier recommendation to turn it
    on is withdrawn.
  - Auto-merge is not used. With required checks on and strict mode off,
    `enable_pr_auto_merge` would merge a PR as soon as its own head is green: out of order,
    outside the lock and without the train.

### 9. CI on train branches (T-2988)
Changes to `.github/workflows/ci.yml`:
```yaml
on:
  pull_request:
  merge_group:
  push:
    branches: [main, merge-train/current, merge-train/shadow]
  workflow_dispatch:
```
- **Only the two train branches trigger.** `merge-train/revert-*` and `merge-train/probe` do not;
  a revert PR gets its `pull_request` CI. T-2988 tests this.
- **The `changes` job is keyed on `github.ref`, not on the event.**
  - When the ref is `refs/heads/merge-train/current` or `refs/heads/merge-train/shadow` (a push,
    or a `workflow_dispatch` on that branch), the base is `git merge-base origin/main HEAD`, which is `M0`: the train's first-parent
    chain starts there. `ci_changes.py` is read from that base (decision 0066), and the files
    come from `git diff -z --no-renames --name-only <base> HEAD`.
  - If the merge-base command fails or the classification fails, run everything (`code=true`),
    as today.
  - **Pull requests: diff from the merge base, not from the base tip** (a bug found on this PR).
    Today the job diffs `"$BASE" HEAD`, with `BASE = github.event.pull_request.base.sha`. A
    docs-only PR whose branch is behind `main` was classified `code=true`: a622e22 touched only
    `docs/` and ran with `run_heavy=true`. Decision 0121 leaves branches behind until they hold
    the lock, so docs-only PRs would often pay for the full CI. T-2988 diffs only the PR's own
    changes:
    - `git merge-base --all "$BASE" "$PR_HEAD"`, with `PR_HEAD = github.event.pull_request.head.sha`;
      **exactly one base is required**. With several bases (a criss-cross history), or none, or a
      failed command, the job sets `code=true`;
    - the files come from `git diff -z --no-renames --name-only "$MB" "$PR_HEAD"`, where `MB` is
      that single base;
    - `ci_changes.py` is still read from `BASE` (decision 0066).

    A docs-only classification only skips jobs on the PR's own CI. When the PR merges through the
    train, the train's own CI classifies the whole combined diff, so it remains the gate.

    Diffing from the PR head, rather than from `HEAD`, which in a `pull_request` checkout is
    GitHub's test merge, makes the result independent of which commit the checkout holds. T-2988
    first confirms the mechanism, for example that `base.sha` lags the test merge's first parent,
    and records it in the PR. A `ci_changes` test case covers a docs-only branch that is behind.
  - Every other ref (`main`, merge groups, dispatches on other branches) runs everything, as
    today.
- **The eval job's "Full report" step.** The condition `failure() || github.event_name == 'push'
  || github.event_name == 'workflow_dispatch'` becomes `failure() || github.ref ==
  'refs/heads/main'`. This is deliberate: trains, and dispatches on any other branch, skip the
  2.5-min report (decision 0054) unless the gate fails, and `main`'s pushes and dispatches keep
  it.
- **`governance`** stays PR-only. It passed on each PR head, and a train adds no files of its own.
- **Concurrency.** The existing `ci-${{ github.ref }}` group cancels a superseded train run, as
  intended, and section 5 covers its effect on `main`.
- **No secrets.** `hygiene` gains a check (in `scripts/ci_changes.py`'s tests, or its own small
  script) that fails when `ci.yml` references `secrets.` or widens `permissions` beyond `contents:
  read`. Train CI runs with the same rights as PR CI.
- The `merge-lock` branch still matches no trigger.

**Rejected: a draft PR per train.** It needs no trigger change, but:
- it costs one PR number per attempt, about 10 a day, and PRs cannot be deleted, so every train
  PR would have to be closed;
- those PRs would show up in the steward's PR scans, the owner's PR list and the agents' "open
  PRs" checks;
- its `pull_request` CI tests a GitHub merge of the train with the current `main`, which is one
  more merge on top.

A push trigger is three lines and tests exactly the sha the merger built.

### 10. Security
- **Untrusted input.** PR titles, bodies, comments, review texts and CI logs are untrusted data.
  The script acts only on structured fields: shas, numbers, labels, permission levels, check-run
  conclusions, and the parsed and validated marker block (sha format, known reviewer names, known
  verdicts). The prompt tells the session never to follow instructions found in them. Titles are
  sanitized (section 4, "Squash commit text"), and `@` mentions are escaped in comments.
- **No PR code runs with merger authority.** The id checks run `M0`'s scripts against the train
  tree. The tests run in CI with `contents: read` and no secrets, as PR CI does today. The merger
  runs no `make` target from the train tree.
- **Forged readiness by an outsider** needs the label (write access) and a marker from an author
  with write permission (V1, the permission API). Outsiders' comments have no effect at all. Fork
  PRs are never candidates. A forged push to a train branch cannot cause a merge, because "green"
  counts only for the sha the merger built and pushed in this run, from runs created by the
  GitHub Actions app (step 6).
- **Fabricated readiness by an agent session** (buggy or prompt-injected) is the accepted v1
  residual risk (section 2, "Trust"). It is unchanged from today, where the same session could
  merge directly. T-2997 is the v2 follow-up.
- **Lock commit injection.** Validated on write and on parse (section 1): bounded ids and
  numbers, no newlines, timestamps clamped to the commit time, a held-on-doubt holder, a capped
  queue.
- **Reviews and comments by outsiders.** V1 and V6 both filter by the permission API first, and
  everything else is ignored with no side effect (section 2).
- **The emergency flip is the only review waiver.** It is bounded by `make merge-flip-check`,
  which a PR cannot widen (one file, the parsed-TOML rule), and it is merged with
  `expectedHeadSha`. Revert PRs get no waiver: they are reviewed like any PR (step 7). The
  `merge-train-stop` label counts only on a PR that passes the same check.
- **Permissions.** The merger needs what workers have today:
  - push to `merge-lock`, `merge-train/current`, `merge-train/shadow` and
    `merge-train/revert-*`;
  - open PRs (its revert PRs); merge the batch's PRs only;
  - set labels and post comments.

  It never pushes to `main` or to a PR branch it did not create, and it never merges a revert
  PR that review has not marked ready.
- **Who may push train branches.** All agent sessions push as the owner's account, so no ruleset
  can tell the merger apart today. The merger's own checks are the safeguard: green counts only
  for the sha it built in this run. The owner can still add a ruleset that restricts creating
  and updating `merge-train/*` to the owner account and blocks other collaborators (Owner steps,
  optional). That becomes a real identity boundary if the merger later runs as a GitHub App
  (Alternatives 4).

### Module placement and interfaces
- `scripts/merge_lock.py` (T-2985): the version-2 format and its validation, the queue,
  `acquire --train/--prs`, exit code 4, `wait`. Make targets `merge-lock-wait PR=<n>`, and
  `merge-lock-status`, which now shows the queue.
- `scripts/gh_client.py` (T-3001): the one GitHub client (`gh api`), with a fake for tests. Its
  operations are PRs, comments with permissions, labels, check runs and workflow runs, git
  commits, merge, and dispatch. It also takes over `merge_lock.py`'s PR-finished read.
- `scripts/merge_ready.py` (T-2986): `mark` / `--carry` (`make mark-ready`), and `verify --pr <n>`
  (V1–V6), plus the marker parser, the reviewer-applicability rule and the "green" rule. These
  are the single definitions that the worker and the merger both import (CLAUDE.md directive 3).
- `scripts/merge_train.py`:
  - T-2987: collect, build and id checks, plus `--dry-run`;
  - T-2994: push, CI wait, the "green" rule applied to trains, the ordered squash of a green
    train with its commit text, pre- and post-squash checks, stop checkpoints, opening the revert
    PR, and recovery;
  - T-2998: red trains: binary search, confirmation, ejection, infrastructure retry, resume
    state;
  - T-2995: `--shadow`, predictions and the shadow tree check, the step-mode fallback, and `make
    merge-train-stats` (the metrics below).
- `make merge-train [DRY_RUN=1] [SHADOW=1]`; `make merge-mode`, which reads
  `origin/main:scripts/merge_train.toml` and fails closed. `make merge-flip-check PR=<n>`
  (T-2996).
- Tests: `scripts/test_merge_lock.py`, `test_gh_client.py`, `test_merge_ready.py` and
  `test_merge_train.py` (stdlib unittest, temporary bare remotes, the fake client), run by `make
  hygiene`.
- No Rust, eval or bindings change.
- If T-2985, T-2986, T-2994 or T-2998 grows past size M while being built, it splits before
  review (planner), along the seams above.

## Decision records and governance
Every implementation slice changes a protected path (`scripts/`, `.github/workflows/`,
`.claude/skills/`, `docs/process/`, `Makefile`). Each slice's PR therefore carries its own
decision record, made with `make new-decision`, and needs explicit approval from
`reviewer-architecture` and `reviewer-security` (steward skill "Governance failures").

This spec PR changes no protected path, so it adds no record. Each record is written by the slice
that puts its rule in force. That keeps records from describing mechanisms that do not exist yet,
and each record cites the section of this spec it implements rather than copying it.

- **T-2985: "The merge lock serves waiters first-in, first-out."** It amends 0075 and 0097
  (section 1): FIFO service, `wait`, rejoining at the tail, validation, train locks and exit 4.
- **T-3001, T-2986, T-2987, T-2994, T-2998, T-2995, T-2988:** one record each for its mechanism. Each says
  that nothing is in force until `mode` says so.
- **T-2996: "Merges go through the merge train when the switch says so."** It is the policy
  record. It amends:
  - **0028.** A PR head counts as "tested against current `main`" when it was merged into a train
    built on the current `main`, together with the PRs before it, and that train's tree passed CI
    (section 4, step 6). The PRs are then squash-merged in train order with `expectedHeadSha`
    while the lock is held. The tree equality is guaranteed by construction and **checked
    detectively**: GitHub guards only the head, so every squash is compared with GitHub's test
    merge before it and with `main` after it. For a mismatching squash that landed, the merger
    opens a revert PR and files a P0, and the revert is reviewed and merged under the normal flow
    (sections 4 and 5). The untested intermediate prefixes are accepted (section 5). The
    readiness verdicts are an
    attestation by the marking session, not a verification (section 2, "Trust"), exactly the
    trust the per-PR flow places in the merging session today. The recommendation to turn on
    "require branches to be up to date" is withdrawn.
  - **0075.** In `train` mode the lock is held by the merger for a batch (`pr: train`). Workers
    hand over with the readiness marker, an attestation as above, and do not take the lock. In
    `lock` and `shadow` mode, 0075 applies as amended by T-2985's record.
  - **0050.** In `train` mode the steward marks orphaned PRs ready instead of merging them. In
    `lock` mode it may merge or unlabel ready PRs without the 24 h clock (section 8).
  - **0121.** Unchanged in substance.
  - **0053.** Still dormant. If the queue becomes available, it replaces the train and the lock.

  It also records the switch rules (section "The switch"):
  - the mode is read from `origin/main` and fails closed;
  - a flip PR that passes `make merge-flip-check` may be merged by any agent without reviewers,
    and `check_governance.py` waives only the decision-record rule for exactly that diff;
  - the `merge-train-stop` label, honoured only on an open PR that passes `make merge-flip-check`;
  - revert PRs opened by the merger are reviewed and merged under the normal flow, with no waiver;
  - the flips to `shadow` and `train` stay fully gated.
- **T-2989, T-2991:** short records for setting `shadow` (with the texts) and `train` (with the
  shadow evidence).

## Alternatives considered
1. **The FIFO lock alone (T-2985).** It fixes starvation, but not the ceiling of about 10/h, the
   extra CI run per PR, or stranded PRs: a worker still has to stay alive until its turn. It is
   kept as the first slice.
2. **GitHub merge queue (0053).** Not available to a personal-account repository (0075).
3. **Move the repository to an organization.** That is the owner's decision. It would revive 0053
   and retire this design (Open questions).
4. **A merger as a scheduled GitHub Actions workflow.** Deterministic, with no model session.
   Rejected for now:
   - pushes and merges made with `GITHUB_TOKEN` trigger no workflows, so neither the train's CI
     nor `main`'s per-merge CI (0054) would run without a PAT or App credential, a long-lived
     write secret in Actions;
   - scheduled workflows start late under load, often by 10–60 min;
   - the lock and marker logic would be duplicated in another runtime.

   This is the natural v2 if routine usage turns out too costly.
5. **A routine every 15 min, without polling inside a run.**
   - Latency: a median wait of about 7.5 min for the next run, plus about 6–7 min of train. That
     is about 14 min at p50 and about 22 min at p90.
   - Cost: 96 sessions a day.
   - The chosen design (a 30-min cadence, polling every 2–3 min over a 40-min window) gives p50/p90
     of 8.7/12.7 min at 3.8/h with p = 0.09, with 48 sessions a day (section 3's table).

   Rejected: it is slower and costs twice the sessions.
6. **A draft PR per train for CI.** Rejected in section 9.
7. **Fast-forward `main` to locally built squash commits.** A direct push to `main`, which every
   prompt forbids. The PRs would show as closed rather than merged. Rejected.
8. **Merge the train branch itself into `main`.** One merge commit per batch breaks "one squash
   commit per task" and leaves the PRs unmerged. Rejected.
9. **Auto-merge for the squash step.** No ordering, merges outside the lock, and no train once
   checks are required (section 8). Rejected.
10. **Rebase PR branches onto the train.** Pushed branches are never rewritten (steward "Branch
    updates"), and a rebase invalidates the approved head. Rejected.
11. **The merger renumbers colliding ids.** Rejected for v1 (section 8).
12. **Every worker runs the merger itself (no routine).** It puts the wait back into the worker's
    budget and brings back stranding at the end of a session. Kept only as the stall fallback.
13. **Bisection one PR at a time, or by halves without a binary search.** Up to n runs, or more
    than ⌈log₂ n⌉ + 2. Rejected for the binary search of section 6.
14. **Speculative pipelined trains.** Not needed yet. The ceiling:
    - per batch, the serial work is the build (about 1 min), CI (4.9 min), and per PR the
      pre-squash wait, the merge call and the checks (about 0.5–0.8 min);
    - a full batch of 5 therefore takes about 9–10 min, which gives a ceiling of about **30
      merges/h** when every train is green;
    - a red train of 5 adds at most 4 runs, about 20 min, so each red train an hour removes about
      a third of that hour's capacity.

    With polling, batches stay small (section 3's table: 1.2 PRs at 3.8/h, 1.5 at 7/h), so the
    effective capacity is lower. With p = 0.09 the lock is busy 34–45% at 3.8/h and 56–71% at
    7/h, and latency has its knee between 7/h and 10/h. **Revisit** when arrivals stay at
    about 8–10/h, or when p90 latency in the "over 6/h" bucket passes 30 min.
15. **The queue on its own ref.** Two refs cannot be updated by one CAS. Rejected.
16. **FIFO by PR number or by CI-green time.** PR number ignores readiness, and CI-green time is
    not stored stably. The marker's server-side `updated_at` is both. Chosen.
17. **Exempting `main` from `cancel-in-progress`** to test every prefix. Multiplies `main`'s CI
    cost by n. Rejected (section 5).

## Test plan
- **`test_merge_lock.py`:**
  - FIFO order; a holder that releases and re-acquires goes behind a waiter;
  - a free lock reserved for the head refuses others;
  - a waiter that stops refreshing expires after `WAITER_TTL`;
  - an abandoned head expires 2 min after `freed`;
  - a head whose PR finished is dropped, and the PR-finished read is rate-limited;
  - a train lock is finished only when its non-empty `prs` are all finished;
  - **a train lock with empty `prs` is never finished, only stale after the TTL**;
  - racing writers: exactly one wins;
  - **a renewal or release that loses its CAS to a waiter refresh retries and succeeds; one whose
    lock was taken over exits 4**;
  - `wait` returns 0, 3 or a timeout;
  - version-1 commits parse;
  - renewal keeps the queue order;
  - **newline injection** in a holder id or PR value is refused on write and dropped on parse;
  - **future timestamps are clamped to the commit's committer time, never to the read time**; an
    out-of-range waiter timestamp expires that waiter;
  - **a commit with a malformed or missing holder line reads as held** (stale after the TTL),
    never free;
  - the queue cap.
- **`test_gh_client.py`:** the fake and the real client agree on request shapes for every
  operation (recorded fixtures).
- **`test_merge_ready.py`:**
  - the marker round trip;
  - an outsider's marker (no write permission) is ignored with no side effects;
  - the newest qualifying marker wins, and an edited comment re-keys FIFO;
  - a bad sha or an unknown reviewer is rejected;
  - excel-fidelity is required exactly for the review skill's paths, and **the list in
    `merge_ready.py` matches `.claude/skills/review/SKILL.md`**;
  - **an outsider's `CHANGES_REQUESTED` review is ignored with no side effects** (V6); a
    collaborator's review after the marker unlabels;
  - pending CI is skipped and red CI is unlabelled;
  - the "green" rule: skipped jobs only with `run_heavy=false`, cancelled runs never green, check
    runs from other apps ignored;
  - `--carry` accepts a clean merge of `main` and refuses an edited one;
  - a stale head invalidates;
  - task-only PRs skip the task check.
- **`test_merge_train.py`** (the fake client, temporary remotes):
  - FIFO by marker time, and the batch cap;
  - deferral on an intra-batch conflict or dependence, ejection on a conflict with `main`;
  - **independence with a criss-cross history** (two merge bases);
  - an id collision ejects the later PR, a collision with `main` ejects that PR, and the train is
    rebuilt after an ejection;
  - **property test:** for random independent PR histories, criss-cross and rename/delete edits
    included, simulating each squash with `git merge-tree --write-tree` onto the previous squash
    gives `tree(T_k)` at every k. With a dependent pair the property can fail, which is why the
    check exists;
  - the write policy: each mode's writes match the table in section 4 exactly (asserted on the
    fake client), and `lock` mode makes none;
  - **mode flips mid-batch:** `origin/main`'s `merge_train.toml` changes to `lock`, or a
    `merge-train-stop` label appears on a PR that passes `merge-flip-check`. This happens right
    after acquisition, before the CI wait, during it, or between two squashes. In each case
    nothing more merges and the lock is released, and the case right after acquisition pushes
    nothing. A malformed or unknown mode reads as `lock`;
  - a `merge-train-stop` label on a PR that fails `merge-flip-check` is ignored, and removed with
    a comment;
  - the revert path: a post-squash mismatch pushes `merge-train/revert-<sha>`, opens the revert
    PR, files the P0 in a separate PR and releases the lock. The merger never merges the revert
    PR;
  - the id checks run `M0`'s scripts with `python -I`, and a planted `tasks.py` or module in the
    train tree is not imported;
  - a shadow push that loses its lease skips the round; shadow predictions and the shadow tree
    check are recorded; the agreement rules score scripted PR histories as specified;
  - binary search against a scripted CI oracle (T-2998):
    - one culprit at each position, **including `P_5` of 5 (exactly 5 runs, `P_1..P_4` merged)**;
    - **a 5-PR red train with the culprit at `P_4` and at `P_5`**, with CI at the 10-min timeout.
      It is cut off by the gate, the known-green prefix is merged before the release, and the
      next hold completes it by resume. The total is ≤ 5 runs, with no third rebuild;
    - the gate's n is the number of PRs each run can merge (full train, `mid`, 1);
    - two culprits;
    - an interaction failure;
    - a non-reproducing failure (no ejection, a P1 task filed);
    - an infrastructure failure (no ejection);
    - a bisection cut off by the hold budget, resumed from the recorded state when `main` is
      still `bisect-base` and the heads are unchanged, and restarted otherwise (with the reason
      recorded);
    - in every case: only culprits are ejected, every innocent PR merges, and no untested tree
      lands;
  - **`main` moves between two squashes:** the pre-squash check fails (its first parent is not the
    expected `main`) and the sequence stops; and when the move falls between the check and the
    squash, the post-squash check fails, a revert PR is opened (not merged) and a P0 filed;
  - the pre-squash merge ref is not recomputed within 2 min: stop;
  - lock renewal exits 4 before step 7 or before a squash: nothing more merges;
  - the hold budget: no CI wait starts with under `CI_TIMEOUT + n × MERGE_BUDGET` left; the lock
    wait is capped at the window's remaining time;
  - the exit handler releases the lock on an error;
  - a partly merged batch, `main` moved before the first squash, a head pushed during CI;
  - the squash text: trailers parsed, de-duplicated, malformed ones dropped; titles sanitized,
    including `@`, `#n` and closing keywords;
  - the step mode: only closed-enum actions with script-generated fields are printed, and a
    deviating state read back aborts;
  - dry-run makes no writes; shadow makes no PR-visible writes beyond the closed-PR housekeeping,
    and takes no lock;
  - shadow reuses T-2998's binary search on `M0` to emit `defer` and `eject` predictions.
- **`make merge-flip-check` and the governance waiver (T-2996):** accepted for a one-line
  `mode = "lock"` change. Refused for:
  - an extra file or an extra line;
  - `mode = "train"` or `"shadow"`;
  - a symlink or a type change;
  - a duplicate key, or a nested `mode`;
  - a rename;
  - a head that moved after the check (the merge with `expectedHeadSha` fails).

  The waiver silences only the decision-record rule.
- **CI (T-2988):**
  - `ci_changes` unit cases for:
    - the train base, and a failed merge-base (`code=true`);
    - a docs-only PR branch that is behind `main` (classified docs-only);
    - a PR with several merge bases (`code=true`);
  - the trigger: pushes to `merge-train/revert-*` and `merge-train/probe` start no run;
  - the secrets/permissions lint;
  - a live check, run **while holding the merge lock** so no merge races it: a docs-only train and
    a code train pushed to `merge-train/current` run, respectively, only `changes`, `hygiene` and
    the rust summary, or everything; neither runs the full report; and a `workflow_dispatch` on
    the branch is classified the same way.
- **Live rollout:** T-2991's shadow evidence (Rollout).

## Rollout
Each slice is one S/M task with its own PR and decision record. Until T-2989 sets `shadow`, every
slice is inert. Until T-2991 sets `train`, nothing merges through the train.

| order | task | what | depends on | reversible by |
|---|---|---|---|---|
| 1 | T-2985 (M) | fair FIFO lock, `make merge-lock-wait`; skills say "wait" | T-2984 | revert (v1 commits stay compatible) |
| 2 | T-3001 (S) | `scripts/gh_client.py` + fake; moves the PR-finished read; **probe of `gh api` writes and `GET /git/commits` from a routine session, answer recorded in this spec** | T-2985 | revert |
| 3 | T-2986 (M) | `merge_ready.py`, `make mark-ready`, labels | T-3001 | revert (nothing reads it yet) |
| 4 | T-2988 (S) | CI on `merge-train/current` and `merge-train/shadow`; classification of trains, and of PRs from a single merge base; report only on `main`; secrets lint | T-2984 | revert the trigger |
| 5 | T-2987 (M) | collect, build, independence, id checks (`--root`), `--dry-run`, property test; `merge_train.toml` = `lock`, `make merge-mode` | T-2986 | revert |
| 6 | T-2994 (M) | push, CI wait, green rule for trains, ordered squash of a green train with its commit text, pre- and post-squash checks, stop checkpoints (including right after acquisition), opening the revert PR, recovery, lock use | T-2987, T-2988 | revert (mode `lock`) |
| 7 | T-2998 (M) | red trains: binary search with the per-wait gate, merging the known-green prefix on a cut-off, required resume, confirmation, ejection, infrastructure retry (section 6) | T-2994 | revert (mode `lock`) |
| 8 | T-2995 (M) | shadow mode on `merge-train/shadow` (predictions, agreement, shadow tree check); the step-mode fallback (only if T-3001 found writes fail); `make merge-train-stats` | T-2998 | revert (mode `lock`) |
| 9 | T-2996 (S) | the policy record; the merger routine row and prompt and the worker and steward clauses in routines.md; `make merge-flip-check` and the `check_governance.py` waiver | T-2995 | revert |
| 10 | T-2989 (M) | work-loop, steward, review and agent-workflow texts for each mode, the emergency-flip procedure, steward daily thresholds; mode = `shadow` | T-2996 | set mode `lock` |
| 11 | owner | create the merger routine; add the required checks with strict mode off; after T-2989 merges, update the worker and steward routines | T-2996, T-2989 | disable the routine |
| 12 | T-2991 (S) | after ≥ 24 h of shadow runs (evidence below), mode = `train` | T-2989; owner routine (blocker, decision 0013) | set mode `lock` |
| 13 | T-2990 (S) | after 7 days in `train` mode: metrics, the first live trains, a decision on the per-PR path | T-2991 | — |
| v2 | T-2997 (M, P3) | per-reviewer verdict records bound to the head sha | T-2989 | — |

T-2985 and T-2988 depend only on this spec (T-2984), and can start as soon as it merges.

**Shadow evidence for T-2991.** It comes from the GitHub API, not from branch history, because
force-pushed branches keep none:
- `gh run list --workflow ci.yml --branch merge-train/shadow --created ">=<date>" --json
  headSha,conclusion,createdAt` lists the shadow CI runs;
- each run's `headSha` is read through the commits API (`gh api
  repos/steveash/rustsheet/commits/<sha>`) to get the batch and the predicted outcome from the
  commit message.

*As built (T-3076, decision 0167):* the prediction is a `[skip ci]` commit with no run of its own,
so `make merge-train-stats` finds the prediction commits through the activity API (every push to
`merge-train/shadow`, force pushes included) and reads each through the commits API; the runs
supply the trains and the CI figures.

The bar, computed by `make merge-train-stats` (T-2995):
- at least 10 scored shadow trains over at least 24 h, the newest within the last 2 h;
- at least 90% of scored predictions **agree**, by the rules in section 4 ("When a shadow
  prediction agrees"), and every disagreement is explained in T-2991's PR;
- **zero `tree-check: mismatch`** records from the shadow tree check (section 4, "Shadow");
- a load bar: shadow roughly doubles merge-stage CI, so PR CI p90 wall time during shadow is no
  more than 20% above the 48 h before shadow started, and job queue delay p90 is under 1 min.

**During shadow, also track:** PR CI wall time, job queue delay (whether train runs delay PR
runs), and the red-train rate against the p = 0.09 estimate.

### Owner steps
1. Create the routine **rustsheet merger** in claude.ai → Code → Routines:
   - repository `steveash/rustsheet` attached, with GitHub access;
   - "new session each run";
   - model Sonnet 5.5;
   - schedule `5,35 * * * *` (UTC);
   - notifications off.

   The prompt is "Prompt — merger" in `docs/process/routines.md`.
2. In the `main` ruleset, add the required status checks `changes (docs-only?)`, `hygiene
   (ratchets, deny, machete, typos, governance)`, `rust (fmt, clippy, test)`, `eval
   (Excel-compatibility gate)` and `python (tools + bindings)`. Leave **"Require branches to be up
   to date before merging" off** (section 8). Leave "Allow auto-merge" off.
3. **After T-2989 merges** (the skills then match the prompts), update the two worker routines and
   the steward routine to the prompts in routines.md, which T-2996 added.
4. *Optional:* add a ruleset that restricts creating, updating and deleting `merge-train/*` to the
   owner account, so that no other collaborator can push train branches (section 10, "Who may push
   train branches"). It adds little while every agent pushes as the owner. It matters once the
   merger has its own identity.

**The prompts** of the merger routine, and the worker and steward merge clauses, live only in
`docs/process/routines.md` (T-2996, decision 0157). The procedure itself lives in the skills
(T-2989).

### Metrics
T-2990 measures these against the 48 h before 2026-10-25. `make merge-train-stats` (T-2995)
computes them from the GitHub API.

**Latency** is measured from the same start event before and after the switch: the first green
`pull_request` CI run on the head that merged. It is bucketed by the arrival rate of the hour
(under 3/h, 3–6/h, over 6/h). The latency target applies **per bucket**, to the "under 3/h" and
"3–6/h" buckets. The "over 6/h" bucket is reported, and its p90 passing 30 min is the revisit
trigger of section 3.

**Simulated figures** in this section and in section 3 come from the performance review's
simulation (`sim2.py` and `sim3.py` in that review session's scratchpad, not committed). Every
figure states the breakage rate p it assumes.

| metric | before | target |
|---|---|---|
| merges per hour (mean / peak hour) | 3.8 / 7 | no lower; the peak is not capped |
| first green on the final head → merged, p50 / p90 (per bucket, under 3/h and 3–6/h) | 24 / 135 min | ≤ 15 / ≤ 30 min when the measured p ≤ 0.10; above that, reported against p |
| **merge-stage CI runs per merged PR** (runs after a PR is ready: under-lock runs before, train runs after) | ≈ 1.05 plus repeat rounds | ≤ 0.85 when the measured p ≤ 0.02 (simulated 0.85 at p = 0); ≤ 1.1 at p ≈ 0.09 (simulated 1.05–1.10 at 3.8/h and 0.92–0.97 at 7/h, confirmation run included), i.e. within the before value once its repeat rounds are counted |
| CI runner-minutes per merged PR (all jobs, PR and train runs) | measured | no higher |
| lock wait for the remaining lock users, p90 | 30–40 min observed | ≤ 15 min |
| PRs stranded (marked ready, unmerged after 2 h, not ejected) | e.g. #1826 | 0 |
| red-train rate; bisection runs per day; ejections by reason | — | reported; red rate compared with 1 − (1 − p)^n |
| deferral rate; effective batch size (mean, p90) | — | reported |
| tree-check failures; `main` red after a train | — | 0; no more than before |
| share of time the lock is held | 60% | reported (simulated 34–45% at 3.8/h and 56–71% at 7/h, p = 0.09) |

**Daily thresholds (steward, T-2989).** Each daily run computes p90 latency, stranded PRs, the
red-train rate and the bisection runs with `make merge-train-stats`. Any of the following
triggers the emergency switch to `lock` (section "The switch") and a P0:
- p90 latency over 60 min on any day;
- more than 2 trains in a day needing more than 6 CI runs;
- a stranded PR;
- any tree-check failure.

### Rollback
- **Fast:** any agent opens the one-line flip PR that sets `mode = "lock"`, labels it
  `merge-train-stop`, runs `make merge-flip-check`, and merges it under the fair lock with
  `expectedHeadSha` once its CI is green. It needs no reviewers (D2). The full CI runs, about
  5 min, because the PR changes `scripts/`. The total delay is under about 10 min (section "The
  switch"). The agent files the P0 in a separate task-only PR. Workers fall back to the per-PR
  flow at their next task. A live train stops at its next checkpoint, and the merger reads
  `lock` at its next poll and stops, so no owner action is needed. PRs still labelled `ready-to-merge` may be merged or
  unlabelled by the steward at once (section 8).
- **Full:** revert T-2989 and T-2996 (texts and prompts). The owner disables the merger routine
  and restores the worker and steward prompts from routines.md. The CI trigger, the scripts and
  the fair lock can stay: they are inert in `lock` mode, and the fair lock helps there too.

## Open questions for the owner
1. **GitHub writes from a routine session.** Answered by T-3001 (section 4, "T-3001's probe"):
   reads and branch pushes work; `gh api` writes were refused by the session's permission layer,
   and `collaborators/{user}/permission` by the proxy, so sessions write through the step mode and
   check marker authors with `author_association`. Owner: allowing `gh api` writes in routine
   sessions would let the merger write directly; until then the step mode applies.
2. **Strict "up to date" check.** Confirm it stays off when the required checks are added. The
   train cannot merge PRs 2..n of a batch with it on.
3. **Renumbering by the merger.** Should the merger renumber a colliding task or decision file
   itself when the PR's branch has been idle 2 h or more (no live session), instead of ejecting
   it? That saves a resume, at the cost of an unreviewed push to that branch.
4. **Batch size.** The cadence sets the batch size, not the cap. With 2–3-min polling, a batch
   holds the PRs that became ready since the last one: about 1.2 at 3.8/h and 1.5 at 7/h. The
   cap of 5 matters only when a backlog builds up (an outage, or a long bisection). A larger cap
   saves CI on a backlog and costs more bisection runs.
5. **Organization move.** If the repository moves to an organization, the GitHub merge queue
   (0053) makes this design and the lock unnecessary.
