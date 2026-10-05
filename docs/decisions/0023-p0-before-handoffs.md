# 0023 — P0 tasks come before handoffs in the queue

- **Status:** accepted (2026-10-05, owner request; T-0036)
- **Context:** `make next-task` ordered handed-off tasks first (decision 0018), then milestone and
  priority. On 2026-10-05 the only P0, T-0931 (deeply nested formulas overflow the stack; prime
  directive 5), sat behind the T-0125 handoff (P2), so the opus worker would have spent a session
  on a P2 before the P0.
- **Decision:** the queue order is: P0 tasks (any milestone), then unresumed handoffs, then
  milestone, priority and id as before. A handoff is still resumed before any non-P0 work, and its
  branch still does not make its lane busy.
- **Also (same PR):** decision numbers collided on 2026-10-05 (T-0032 and T-0033 both wrote 0021
  in parallel PRs). The status-log record is renumbered 0024, and `scripts/check_governance.py`
  (run by `make hygiene` and CI) now fails when two records share a number. CI tests the PR merged
  with main, so the second PR to land sees the clash.
- **Consequences:** a P0 is never delayed by an escalation. The worker prompts need no change: the
  opus worker's first `make next-task TIER=opus` returns the P0 (it is not a handoff, so the worker
  takes it through the lane walk, and P0 tasks have so far always sat in critical lanes). This
  supersedes the "handoffs first" ordering clause of decision 0018.
