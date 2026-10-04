---
name: review
description: Run RustSheet's multi-agent code review on the current branch versus main - security, architecture, quality/duplication, Excel fidelity and (when relevant) performance reviewers in parallel - and consolidate verdicts. Use before opening or merging any PR.
---
# Multi-agent review

1. Make sure the branch is committed and `make check` passed (reviewers are not linters).
2. Collect context: `git diff --stat origin/main...HEAD`, the task id (branch name) and its file.
3. Launch these subagents **in parallel** (one message, multiple Agent calls), each with a
   prompt containing: the task id, task file path, spec path(s), and "review `git diff
   origin/main...HEAD`":
   - `reviewer-security` (always)
   - `reviewer-architecture` (always)
   - `reviewer-quality` (always)
   - `reviewer-excel-fidelity` (if the diff touches crates/rustsheet-engine, crates/rustsheet-xlsx,
     eval/suites, or eval/baseline)
   - `reviewer-performance` (if the diff touches evaluation, storage, parser, graph, structural
     edits, or bulk APIs)
   Each reviewer runs on the model set in its agent frontmatter (decision 0015: security,
   architecture and quality on Sonnet; Excel fidelity and performance on Opus). Do not override it
   per call, with one exception: the steward's weekly spot-audit launches every reviewer with
   `model: opus`, so a cheaper reviewer's miss is re-checked by the stronger model.
4. Consolidate into a table: reviewer | verdict | blocking findings. A PR is mergeable only when
   every reviewer returned `APPROVE` (MINOR findings may be deferred as follow-up tasks).
5. For each MINOR finding you choose not to fix, create a follow-up task or note why it is moot.
6. Paste the table into the PR body ("Agent review" section).

Reviewers must not see your reasoning about why the code is fine — give them only the task
and the diff, so their review is independent.
