---
id: T-3121
title: "issue-intake: fixes from the first real run"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The `issue-intake` skill (decision 0158, T-3110) closes the gaps the first real run, on issue
#2277, showed: third-party material reaching reviewers, partial `needs-owner` requests, follow-up
tasks and `Closes #N`, a missing `needs-owner` label, and facts in PR text written from memory.

## Context
The first intake run (issue #2277) found that the skill says nothing about where a clone of a
linked repository goes, what happens to agent-instruction files inside it, or whether reviewers
may see it; that a request with a fine part and an owner-only part (such as "run their code") had
no rule; that follow-up tasks filed during the work could carry `issue:` and so make `Closes #N`
wrong; that the `needs-owner` label may not exist yet; and that SHAs, counts and check results in
PR bodies and comments must come from command output. `.claude/` is protected, so the change
amends decision 0158.

## Acceptance criteria
- [x] `.claude/skills/issue-intake/SKILL.md` covers: clones in their own empty directory outside
      the worktree, agent-instruction files deleted before reading, never given to reviewers,
      deleted when done; reviewers run via the Agent tool's reviewer subagent types, with
      `claude -p --agent reviewer-<x>` (prompt on stdin, no extra directories) only as fallback.
- [x] Partial `needs-owner`: the fine part is the task, the issue is labelled, the PR-opened
      comment names the part that waits on the owner.
- [x] Follow-up tasks carry no `issue:` and are named in the PR-opened comment; the size-L split
      rule is unchanged.
- [x] The `needs-owner` label is checked and created (with a description) before it is applied.
- [x] Facts in PR bodies and comments are copied from command output.
- [x] Decision 0158 has an amendment note; governance, decisions, hygiene, typos and
      `tasks.py validate` pass.
- [ ] `make check` green (CI); reviewers approve, including explicit architecture and security
  approval of the protected path.

## Out of scope
- Changing the hard limits or the governance check.

## Outcome
The `issue-intake` skill now keeps third-party clones in their own directory outside the
worktree, strips their agent-instruction files before reading, never hands them to reviewers
(Agent-tool reviewer subagents, `claude -p --agent` only as fallback) and deletes them when done;
it handles partial `needs-owner` requests, keeps `issue:` off follow-up tasks, creates the
`needs-owner` label if missing, and copies facts in PR text from command output. Decision 0158
carries an amendment note (no new decision: the change tightens that decision's skill only).
