"""Tests for merge_janitor.py (`make janitor`, decision 0224): the decisions, and one pass against
gh_client's fake with fake git facts and effects."""

from __future__ import annotations

import sys
import unittest
import urllib.parse
from datetime import UTC, datetime, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_janitor  # noqa: E402 (the sys.path line above must run first)
import merge_ready  # noqa: E402
from gh_client import FakeGh, Reply  # noqa: E402
from merge_catchup import Refused  # noqa: E402
from merge_janitor import CATCH_UP, MARK, ORPHANED, REFUSED, RENUMBER, UNLABEL, State  # noqa: E402

NOW = datetime(2026, 11, 1, 12, 0, tzinfo=UTC)
HEAD, OLD, PUSHED = "a" * 40, "b" * 40, "c" * 40
PAGE = "per_page=100&page=1"
EJECTED, READY = merge_ready.EJECTED_LABEL, merge_ready.READY_LABEL


def hours(n: float) -> timedelta:
    return timedelta(hours=n)


def marker(head: str) -> merge_ready.Marker:
    return merge_ready.Marker(head, None, None, {"security": "APPROVE"}, "session-1")


class Facts:
    def __init__(self, ci: str = merge_ready.GREEN, lists_only: bool = True, carries: bool = True) -> None:
        self.state, self.only, self.carry = ci, lists_only, carries
        self.calls: list[str] = []

    def ci(self, state: State) -> str:
        self.calls.append("ci")
        return self.state

    def lists_only(self, state: State) -> bool:
        self.calls.append("lists_only")
        return self.only

    def carries(self, state: State, prev: str) -> bool:
        self.calls.append(f"carries {prev[:1]}")
        return self.carry


def ejected(reason: str, idle: float, *done: str) -> State:
    return State(7, "agent/x/T-0001-x", HEAD, [EJECTED], hours(idle), reason, marker(OLD), set(done))


def stale(idle: float, labels: tuple[str, ...] = (READY,), *done: str) -> State:
    return State(7, "agent/x/T-0001-x", HEAD, list(labels), hours(idle), "", marker(OLD), set(done))


def kind(state: State, facts: Facts | None = None) -> str | None:
    action = merge_janitor.decide(state, facts or Facts())
    return action.kind if action else None


class Decide(unittest.TestCase):
    def test_an_id_collision_is_renumbered_after_an_hour(self) -> None:
        reason = "id collision with T-0001; renumber per decision 0121 (2)"
        self.assertIsNone(kind(ejected(reason, 0.5)))
        self.assertEqual(kind(ejected(reason, 1.5)), RENUMBER)

    def test_a_baseline_only_conflict_is_caught_up(self) -> None:
        self.assertEqual(kind(ejected("conflict with main", 1.5)), CATCH_UP)

    def test_a_real_conflict_waits_then_is_orphaned_once(self) -> None:
        facts = Facts(lists_only=False)
        self.assertIsNone(kind(ejected("conflict with main", 2), facts))
        self.assertEqual(kind(ejected("conflict with main", 4.5), facts), ORPHANED)
        self.assertIsNone(kind(ejected("conflict with main", 9, ORPHANED), facts))

    def test_ci_red_is_orphaned_after_four_hours(self) -> None:
        reason = "train CI red, confirmed on this PR alone on main"
        self.assertIsNone(kind(ejected(reason, 3)))
        self.assertEqual(kind(ejected(reason, 4)), ORPHANED)

    def test_its_own_push_is_marked_once_ci_is_green(self) -> None:
        self.assertEqual(kind(ejected("id collision with 0002", 0.1, RENUMBER)), MARK)
        self.assertIsNone(kind(ejected("id collision with 0002", 0.1, RENUMBER), Facts(merge_ready.PENDING)))
        red = Facts(merge_ready.RED)
        self.assertIsNone(kind(ejected("conflict with main", 1, CATCH_UP), red))
        self.assertEqual(kind(ejected("conflict with main", 5, CATCH_UP), red), ORPHANED)

    def test_a_refused_step_is_not_retried_and_is_orphaned_later(self) -> None:
        facts = Facts()
        self.assertIsNone(kind(ejected("conflict with main", 2, REFUSED), facts))
        self.assertEqual(facts.calls, [])
        self.assertEqual(kind(ejected("conflict with main", 5, REFUSED), facts), ORPHANED)
        self.assertIsNone(kind(ejected("conflict with main", 5, CATCH_UP, REFUSED, ORPHANED), facts))

    def test_a_stale_marker_over_a_carry_merge_is_marked_again(self) -> None:
        facts = Facts()
        self.assertEqual(kind(stale(2.5), facts), MARK)
        self.assertEqual(facts.calls, ["carries b", "ci"])
        self.assertIsNone(kind(stale(2.5), Facts(merge_ready.PENDING)))
        self.assertEqual(kind(stale(2.5), Facts(merge_ready.RED)), UNLABEL)

    def test_a_stale_marker_over_another_change_is_unlabelled(self) -> None:
        self.assertEqual(kind(stale(2.5), Facts(carries=False)), UNLABEL)
        self.assertIsNone(kind(stale(2.5, ()), Facts(carries=False)))  # already unlabelled
        self.assertEqual(kind(stale(2.5, (READY,), REFUSED)), UNLABEL)

    def test_a_stale_marker_waits_two_hours_and_a_current_one_is_left_alone(self) -> None:
        self.assertIsNone(kind(stale(1.5)))
        current = stale(5)
        current.marker = marker(HEAD)
        self.assertIsNone(kind(current))

    def test_an_unknown_idle_time_does_nothing(self) -> None:
        state = ejected("id collision with T-0001", 9)
        state.idle = None
        self.assertIsNone(kind(state))


class Notes(unittest.TestCase):
    def test_round_trip_with_and_without_the_footer(self) -> None:
        body = f"merge janitor: x\n\n{merge_janitor.note_block(HEAD, RENUMBER)}"
        self.assertEqual(merge_janitor.parse_note(body), (HEAD, RENUMBER))
        self.assertEqual(merge_janitor.parse_note(body + merge_ready.FOOTER), (HEAD, RENUMBER))
        self.assertIsNone(merge_janitor.parse_note(body + "\nmore"))

    def test_reason_kind(self) -> None:
        self.assertEqual(merge_janitor.reason_kind("conflict with main"), merge_janitor.CONFLICT)
        self.assertEqual(
            merge_janitor.reason_kind("id collision with T-0001; renumber"), merge_janitor.COLLISION
        )
        self.assertEqual(merge_janitor.reason_kind("train CI red, confirmed"), merge_janitor.CI_RED)
        self.assertEqual(merge_janitor.reason_kind("readiness invalidated by a push"), merge_janitor.OTHER)


class Effects:
    def __init__(self, refuse: str | None = None) -> None:
        self.calls: list[str] = []
        self.refuse = refuse

    def step(self, name: str, state: State) -> str:
        self.calls.append(f"{name} #{state.pr}")
        if self.refuse == name:
            raise Refused("the merge with main conflicts outside the eval-gated baseline lists")
        return PUSHED

    def catch_up(self, state: State) -> str:
        return self.step(CATCH_UP, state)

    def renumber(self, state: State) -> str:
        return self.step(RENUMBER, state)

    def mark(self, state: State) -> None:
        self.step(MARK, state)


def comment(ident: int, body: str, login: str = "owner") -> dict[str, object]:
    stamp = f"2026-11-01T0{ident % 10}:00:00Z"
    return {
        "id": ident,
        "body": body,
        "user": {"login": login},
        "author_association": "OWNER" if login == "owner" else "NONE",
        "created_at": stamp,
        "updated_at": stamp,
    }


def pull(
    number: int, head: str, labels: tuple[str, ...], branch: str = "agent/x/T-0001-x"
) -> dict[str, object]:
    repo = {"full_name": "o/r", "owner": {"type": "User"}}
    return {
        "number": number,
        "draft": False,
        "labels": [{"name": n} for n in labels],
        "head": {"sha": head, "ref": branch, "repo": repo},
        "base": {"ref": "main", "repo": repo},
    }


class Pass(unittest.TestCase):
    """One `Janitor.run` over the fake: PR 7 ejected for an id collision, idle 3 h."""

    def setUp(self) -> None:
        self.pulls = [pull(7, HEAD, (EJECTED,))]
        self.comments: dict[int, list[object]] = {
            7: [
                comment(1, merge_ready.marker_body(marker(OLD))),
                comment(
                    2,
                    "merge train: ready-to-merge removed: id collision with T-0001; renumber per decision 0121 (2)",
                ),
            ]
        }
        self.pushed: dict[int, str] = {7: "2026-11-01T09:00:00Z"}

    def fake(self) -> FakeGh:
        replies = {
            ("GET", f"repos/o/r/pulls?state=open&{PAGE}"): Reply(self.pulls),
            ("GET", "repos/o/r/collaborators/owner/permission"): Reply({"permission": "admin"}),
            ("GET", "repos/o/r/collaborators/stranger/permission"): Reply({"permission": "read"}),
        }
        for p in self.pulls:
            n, head, branch = p["number"], p["head"]["sha"], p["head"]["ref"]  # type: ignore[index]
            replies[("GET", f"repos/o/r/issues/{n}/comments?{PAGE}")] = Reply(self.comments.get(n, []))
            replies[("POST", f"repos/o/r/issues/{n}/comments")] = Reply({"id": 99})
            replies[("DELETE", f"repos/o/r/issues/{n}/labels/{READY}")] = Reply({})
            ref = urllib.parse.quote(f"refs/heads/{branch}", safe="")
            activity = [{"after": head, "timestamp": self.pushed[n]}] if n in self.pushed else []
            replies[("GET", f"repos/o/r/activity?ref={ref}&per_page=100")] = Reply(activity)
        return FakeGh(replies=replies)

    def run_once(self, effects: Effects | None = None, facts: Facts | None = None, dry_run: bool = False):  # type: ignore[no-untyped-def]
        client = self.fake()
        janitor = merge_janitor.Janitor(client, facts or Facts(), effects or Effects(), dry_run)
        errors = janitor.run(NOW)
        self.assertEqual(errors, 0, janitor.log)
        writes = [
            (r.method, r.path.rsplit("/", 2)[-2:], (r.body or {}).get("body", ""))
            for r in client.requests
            if r.method != "GET"
        ]
        return janitor, writes

    def test_an_id_collision_is_renumbered_and_noted_with_the_pushed_head(self) -> None:
        effects = Effects()
        janitor, writes = self.run_once(effects)
        self.assertEqual(effects.calls, ["renumber #7"])
        self.assertEqual(len(writes), 1)
        self.assertEqual(merge_janitor.parse_note(str(writes[0][2])), (PUSHED, RENUMBER))
        self.assertIn("#7: pushed " + PUSHED, janitor.log)

    def test_dry_run_writes_and_pushes_nothing(self) -> None:
        effects = Effects()
        janitor, writes = self.run_once(effects, dry_run=True)
        self.assertEqual((effects.calls, writes), ([], []))
        self.assertTrue(janitor.log[0].startswith("#7: renumber"))

    def test_a_refusal_is_noted_on_the_head_and_not_retried(self) -> None:
        effects = Effects(refuse=RENUMBER)
        _, writes = self.run_once(effects)
        self.assertEqual(merge_janitor.parse_note(str(writes[0][2])), (HEAD, REFUSED))
        self.comments[7].append(comment(3, str(writes[0][2])))
        again = Effects()
        _, writes = self.run_once(again)
        self.assertEqual((again.calls, writes), ([], []))

    def test_the_next_run_marks_its_own_push_once_ci_is_green(self) -> None:
        self.pulls = [pull(7, PUSHED, (EJECTED,))]
        self.pushed[7] = "2026-11-01T11:50:00Z"
        self.comments[7].append(
            comment(3, f"merge janitor: pushed\n\n{merge_janitor.note_block(PUSHED, RENUMBER)}")
        )
        effects = Effects()
        self.run_once(effects, Facts(merge_ready.PENDING))
        self.assertEqual(effects.calls, [])
        self.run_once(effects, Facts(merge_ready.GREEN))
        self.assertEqual(effects.calls, ["mark #7"])

    def test_notes_by_outsiders_do_not_count(self) -> None:
        self.comments[7].append(
            comment(3, f"x\n\n{merge_janitor.note_block(HEAD, REFUSED)}", login="stranger")
        )
        effects = Effects()
        self.run_once(effects)
        self.assertEqual(effects.calls, ["renumber #7"])

    def test_an_orphan_is_commented_once_per_head(self) -> None:
        self.comments[7][1] = comment(2, "merge train: ready-to-merge removed: conflict with main")
        self.pushed[7] = "2026-11-01T07:00:00Z"
        _, writes = self.run_once(Effects(), Facts(lists_only=False))
        self.assertEqual(merge_janitor.parse_note(str(writes[0][2])), (HEAD, ORPHANED))
        self.assertIn("opus run or the owner", str(writes[0][2]))
        self.comments[7].append(comment(3, str(writes[0][2])))
        _, writes = self.run_once(Effects(), Facts(lists_only=False))
        self.assertEqual(writes, [])

    def test_a_stale_labelled_marker_is_unlabelled(self) -> None:
        self.pulls = [pull(8, HEAD, (READY,))]
        self.comments = {8: [comment(1, merge_ready.marker_body(marker(OLD)))]}
        self.pushed = {8: "2026-11-01T09:00:00Z"}
        _, writes = self.run_once(Effects(), Facts(carries=False))
        self.assertEqual([w[0] for w in writes], ["DELETE", "POST"])
        self.assertIn("readiness invalidated by a push", str(writes[1][2]))

    def test_prs_without_a_marker_or_an_ejection_drafts_and_forks_are_skipped(self) -> None:
        fork = pull(9, HEAD, (EJECTED,))
        fork["head"] = {**fork["head"], "repo": {"full_name": "someone/r"}}  # type: ignore[dict-item]
        draft = pull(10, HEAD, (EJECTED,))
        draft["draft"] = True
        self.pulls = [pull(8, HEAD, ()), fork, draft]
        effects = Effects()
        _, writes = self.run_once(effects)
        self.assertEqual((effects.calls, writes), ([], []))


if __name__ == "__main__":
    unittest.main()
