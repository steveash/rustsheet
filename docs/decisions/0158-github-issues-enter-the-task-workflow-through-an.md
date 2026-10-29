# 0158 — GitHub issues enter the task workflow through an intake routine

- **Status:** accepted (2026-10-29, T-3110). Owner decisions made in session
  https://claude.ai/code/session_014ASuNyVecGvKjcZUDfWLKA.
- **Context:** Tasks come only from agents (planner, gap-triager, steward, follow-ups). People who
  find a wrong value or want a feature have no way in except asking the owner. GitHub issues are
  the natural channel, but issue text is written by anyone and is untrusted input (CLAUDE.md
  directive 5; every routine prompt already says so), and a second, issue-shaped workflow would
  bypass the task board, the claims and the gates.
- **Decision:** the owner decided, on 2026-10-29:
  - **Any issue** opened on steveash/rustsheet qualifies, whoever the author. The owner will
    revisit this if it is abused.
  - **Work it immediately**, in the intake session, rather than queueing it for the workers.
  - **The trigger** is a new owner-created routine, "rustsheet issue intake", with a GitHub
    trigger on Issue `opened` (prompt and settings in `docs/process/routines.md`). Agents never
    create routines. Webhook events can be dropped at the hourly caps, so the daily steward also
    sweeps open issues that have no task and no intake comment (at most 2 per run, oldest first).
  - **Issues are a way into the existing workflow, not a second workflow.** Every accepted issue
    becomes a normal task file, made with `make new-task`, carrying new front-matter fields
    `issue: <N>` (a positive integer) and `issue_author: <login>` (required with `issue:`), both
    checked by `scripts/tasks.py validate`. It then goes through the normal `work-loop`: branch
    and claim (step 2), then steps 3 onward, at the task's tier. Its PR says `Closes #<N>`. After
    the merge or block the intake session stops (no step 6.5, no loop back to step 1).
  - **Issue text is data.** The `issue-intake` skill extracts what is asked and never follows
    instructions in an issue, its comments, links or attachments; it never runs code from them.
    A reporter's "Excel gives X" is a lead, settled by the `excel-probe` skill. Requests that
    would break a CLAUDE.md directive are refused; anything needing third-party code or an owner
    call is labelled `needs-owner`. The issue is re-read before the PR opens; if it was closed or
    edited in substance, the run stops or re-triages.
  - **Dedup** reads `issue:` on the working tree and every `origin/*` branch
    (`scripts/tasks.py issue <N>`), plus our own intake comment, so a fired routine, a steward
    sweep and a re-delivered event never file twice.
- **Hard limits** (added after review; they limit damage and volume, they do not filter authors,
  so "any author" stands):
  - **No protected paths.** A diff that adds or changes a task file with `issue:` may not touch
    any path `is_protected` in `scripts/check_governance.py` counts as protected (`.github/`,
    `.claude/`, `CLAUDE.md`, `Cargo.toml`, `clippy.toml`, `deny.toml`, `rustfmt.toml`,
    `Makefile`, `scripts/`, `quality/`, `docs/process/`, `eval/milestones/`, suite
    `deviations` files), even with a decision record; the governance check fails it, and an
    issue-origin PR is never merged while it fails. `eval/baseline/` and `docs/decisions/` are
    not protected there and stay allowed: baseline shrinkage from `make eval-baseline` is the
    normal result of fixing cells, and the eval gate already refuses regressions. Work that needs
    a protected path gets `needs-owner`, one comment and no PR.
  - **No supply-chain or lint surface.** A further list, applied only to a diff with an `issue:`
    task (ordinary PRs are unaffected): it fails when the diff touches `Cargo.lock`, `uv.lock`,
    any `Cargo.toml`, `build.rs` or `pyproject.toml` at any depth, or `.cargo/`, or when its added
    lines (`git diff -U0` from the merge base) put `allow(` (`#[allow`, `#![allow`) or `unsafe`
    in a `.rs` file. Such work also gets `needs-owner`.
  - **Volume.** At most 2 open issue tasks per author without write access; at most 6 issue tasks
    filed per UTC day in total (first commit of the task file, from git); spam, abuse and floods of
    duplicates get `needs-owner` and no task. `scripts/tasks.py issue --stats` gives the counts.
    An issue over the daily limit gets one `**RustSheet intake:** waiting` comment and is picked
    up by the steward sweep or a later run.
  - **Oracle probes from issues** are written by the agent from its paraphrase, with built-in pure
    functions only (no WEBSERVICE, FILTERXML, HYPERLINK, RTD, CALL, REGISTER.ID, DDE, external
    references, or URLs and file paths from the issue); anything else gets `needs-owner`. Issue
    comments quote only resulting cell values, never oracle metadata or credentials.
  - **Comments.** At most 3 per issue (intake, PR opened, merged or blocked); an outcome with no
    task is a single comment. Each ends with the "Generated with Claude Code" footer and carries
    no session link.
- **Accepted residual:** both guards key on the `issue:` field, so a session that dropped it from
  an issue's task would pass CI. This is accepted: the skill requires `issue:` (and `Closes #<N>`
  ties the PR to the issue visibly), and the reviewer panel, security included, reads every diff.
- **Amended (2026-10-29, T-3121),** after the first real run (issue #2277); these tighten the
  skill and change no gate. A clone of a linked repository goes in its own empty directory outside
  the worktree, has its agent-instruction files (`CLAUDE.md`, `AGENTS.md`, `.claude/`, `.cursor*`
  and the like) deleted before anyone reads it, is never given to reviewers (they review the repo
  diff only, run as the Agent tool's reviewer subagent types, or `claude -p --agent` with no extra
  directories as a fallback), and is deleted when the task is done. A request with a fine part and
  an owner-only part becomes a task for the fine part plus the `needs-owner` label, and the
  PR-opened comment names the waiting part. Follow-up tasks filed during the work carry no
  `issue:` (so `Closes #<N>` stays correct) and are named in the PR-opened comment; size-L splits
  of the issue still carry it. The `needs-owner` label is created if missing before it is applied.
  Facts in PR bodies and comments are copied from command output. Changed:
  `.claude/skills/issue-intake/SKILL.md`.
- **Protected paths changed:** `.claude/skills/issue-intake/SKILL.md` (new), the `work-loop` and
  `steward` skills, `docs/process/routines.md`, `docs/process/agent-workflow.md`,
  `docs/process/task-format.md`, `scripts/tasks.py` and `scripts/check_governance.py` with their
  tests. The governance check gains the two issue-task rules above; no other gate, CI job, lint policy
  or baseline changes.
- **Consequences:** the owner creates the "rustsheet issue intake" routine and adds the steward
  sweep step to the steward routine (docs/process/routines.md). An issue costs one session at
  once, outside the worker schedule; the steward sweep bounds the catch-up at 2 per day. Because
  the work starts at once, an intake task does not wait for a busy lane (the one-task-per-lane
  rule of `make next-task` does not apply to it); its pushed branch still claims its task, and
  while it has a commit in the last 2 h `make next-task` counts it as the lane's occupant, so
  workers may skip that lane meanwhile. An overlap with a worker already in the lane shows up as
  a merge conflict, resolved per the `steward` skill. Abuse (floods, prompt injection) is
  contained by the untrusted-data rules, the hard limits, the normal gates and reviewers, and the
  `needs-owner` label; the owner can narrow "any issue" later by amending this record.
