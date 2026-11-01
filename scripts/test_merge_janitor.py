"""Tests for merge_janitor.py (`make janitor`, decision 0224): the decisions, and one pass against
gh_client's fake with fake git facts and effects."""

from __future__ import annotations

import contextlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
import urllib.parse
from collections.abc import Callable
from unittest import mock
from datetime import UTC, datetime, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_janitor  # noqa: E402 (the sys.path line above must run first)
import merge_ready  # noqa: E402
import merge_step  # noqa: E402
from gh_client import FakeGh, Reply  # noqa: E402
from merge_catchup import Refused  # noqa: E402
from merge_lock import GitError  # noqa: E402
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
        # The merger's own reason lines, built by its own functions, match exactly; anything else is OTHER.
        kinds = {
            "conflict with main": merge_janitor.CONFLICT,
            merge_janitor.merge_train.id_collision(
                ["duplicate id T-0001 (also x)", "decision number 0002 used twice"]
            ): merge_janitor.COLLISION,
            merge_janitor.merge_train.id_collision([]): merge_janitor.COLLISION,
            "train CI red, confirmed on this PR alone on main": merge_janitor.CI_RED,
            "readiness invalidated by a push": merge_janitor.OTHER,
            "conflict with main, and reason: ci-red": merge_janitor.OTHER,
            "id collision with T-0001; renumber": merge_janitor.OTHER,
            "train CI red, confirmed": merge_janitor.OTHER,
        }
        for reason, kind in kinds.items():
            with self.subTest(reason=reason):
                self.assertEqual(merge_janitor.reason_kind(reason), kind)


class Effects:
    def __init__(self, refuse: str | None = None) -> None:
        self.calls: list[str] = []
        self.refuse = refuse
        self.filed: list[str] = []
        # Set to see the order of the note and the push.
        self.events: list[str] = []

    def step(self, name: str, state: State, announce: Callable[[str], None] | None = None) -> str:
        self.calls.append(f"{name} #{state.pr}")
        if self.refuse == name:
            raise Refused("the merge with main conflicts outside the eval-gated baseline lists")
        if announce is not None:
            announce(PUSHED)
            self.events.append("pushed")
        return PUSHED

    def catch_up(self, state: State, announce: Callable[[str], None]) -> str:
        return self.step(CATCH_UP, state, announce)

    def renumber(self, state: State, announce: Callable[[str], None]) -> str:
        return self.step(RENUMBER, state, announce)

    def mark(self, state: State) -> None:
        self.step(MARK, state)

    def file_p0(self, log: list[str], title: str, context: str) -> bool:
        self.filed.append(title)
        return self.files

    files = True  # set False: the task-only PR could not be opened


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
            **{("GET", f"repos/o/r/labels/{name}"): Reply({"name": name}) for name in merge_ready.LABELS},
        }
        for p in self.pulls:
            n, head, branch = p["number"], p["head"]["sha"], p["head"]["ref"]  # type: ignore[index]
            replies[("GET", f"repos/o/r/pulls/{n}")] = Reply(p)
            replies[("GET", f"repos/o/r/issues/{n}/comments?{PAGE}")] = Reply(self.comments.setdefault(n, []))
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
        held = pull(11, HEAD, (EJECTED, merge_janitor.HOLD_LABEL))  # an owner's merge-hold
        self.pulls = [pull(8, HEAD, ()), fork, draft, held]
        self.comments[11] = list(self.comments[7])
        self.pushed[11] = self.pushed[7]
        effects = Effects()
        _, writes = self.run_once(effects)
        self.assertEqual((effects.calls, writes), ([], []))

    def test_the_note_is_posted_before_the_push(self) -> None:
        client = self.fake()
        effects = Effects()
        order: list[str] = []
        renumber = effects.renumber

        def watched(state: State, announce: Callable[[str], None]) -> str:
            def note_then(head: str) -> None:
                announce(head)
                order.append(f"noted {len([r for r in client.requests if r.method == 'POST'])}")

            pushed = renumber(state, note_then)
            order.append("pushed")
            return pushed

        effects.renumber = watched  # type: ignore[method-assign]
        merge_janitor.Janitor(client, Facts(), effects).run(NOW)
        self.assertEqual(order, ["noted 1", "pushed"])

    def test_no_catch_up_or_renumber_starts_once_the_budget_is_spent(self) -> None:
        effects = Effects()
        janitor = merge_janitor.Janitor(self.fake(), Facts(), effects, deadline=10.0, clock=lambda: 11.0)
        self.assertEqual(janitor.run(NOW), 0)
        self.assertEqual(effects.calls, [])
        self.assertIn("#7: renumber left for the next run: this run's budget is spent", janitor.log)

    def escalation(self, merged_at: str | None, *notes: str) -> tuple[Effects, FakeGh]:
        """One run with PR 8 marked ready on its current head at 01:00 (11 h before NOW), not
        ejected, its branch's `notes`, and one PR updated in the window, merged at `merged_at`."""
        self.pulls = [pull(8, HEAD, (READY,))]
        self.comments = {8: [comment(1, merge_ready.marker_body(marker(HEAD)))]}
        self.comments[8] += [comment(2 + i, f"x\n\n{note}") for i, note in enumerate(notes)]
        self.pushed = {8: "2026-11-01T00:30:00Z"}
        client = self.fake()
        recent = [{"number": 3, "updated_at": "2026-11-01T11:00:00Z", "merged_at": merged_at}]
        query = "state=all&sort=updated&direction=desc&per_page=100&page=1"
        client.replies[("GET", f"repos/o/r/pulls?{query}")] = Reply(recent)
        effects = Effects()
        janitor = merge_janitor.Janitor(client, Facts(), effects)
        self.assertEqual(janitor.run(NOW), 0, janitor.log)
        return effects, client

    def test_a_long_stranded_pr_with_no_merge_files_one_owner_p0(self) -> None:
        effects, client = self.escalation(None)
        self.assertEqual(effects.filed, ["Merge train: nothing merged in 6.0 h with #8 stranded 11.0 h"])
        [posted] = [r.body for r in client.requests if r.method == "POST"]
        self.assertEqual(merge_janitor.parse_note(str(posted["body"])), (HEAD, merge_janitor.ESCALATED))  # type: ignore[index]

    def test_the_escalation_is_filed_once_per_head_and_not_after_a_merge(self) -> None:
        noted, _ = self.escalation(None, merge_janitor.note_block(HEAD, merge_janitor.ESCALATED))
        self.assertEqual(noted.filed, [])
        merged, _ = self.escalation("2026-11-01T10:00:00Z")
        self.assertEqual(merged.filed, [])

    def test_a_p0_that_could_not_be_filed_is_not_noted_and_is_retried(self) -> None:
        with mock.patch.object(Effects, "files", False):
            failed, client = self.escalation(None)
        self.assertEqual(len(failed.filed), 1)
        self.assertEqual([r for r in client.requests if r.method == "POST"], [])  # no ESCALATED note
        retried, _ = self.escalation(None)  # the next run: nothing on the PR says it was filed
        self.assertEqual(len(retried.filed), 1)

    def test_file_tasks_reports_whether_the_pr_was_opened(self) -> None:
        log: list[str] = []
        with mock.patch.object(
            merge_janitor.merge_train_filing, "fetch_main", side_effect=GitError("no network")
        ):
            self.assertFalse(
                merge_janitor.merge_train_filing.file_tasks(Path("."), FakeGh(), log, [("P0", "t", "c")])
            )
        self.assertEqual(log, ["the tasks were not filed: no network", "unfiled P0: t"])
        self.assertTrue(merge_janitor.merge_train_filing.file_tasks(Path("."), FakeGh(), [], []))

    # Step mode (decision 0169): the janitor's writes go through merge_step.StepClient.
    def perform(self, action: dict[str, object]) -> None:
        """What the session does on GitHub for one step action, on the fake's state."""
        pr = action["pr"]
        if action["action"] == merge_step.COMMENT:
            self.comments[pr].append(comment(50 + len(self.comments[pr]), str(action["body"])))  # type: ignore[index]
        else:
            pull_ = next(p for p in self.pulls if p["number"] == pr)
            pull_["labels"] = [{"name": n} for n in action["labels"]]  # type: ignore[union-attr]

    def stepped(self, dry_run: bool = False, effects: Effects | None = None, facts: Facts | None = None):  # type: ignore[no-untyped-def]
        seen: list[dict[str, object]] = []

        def session(action: dict[str, object]) -> str:
            seen.append(action)
            self.perform(action)
            return merge_step.DONE

        inner = self.fake()
        client = merge_step.StepClient(inner, session, sleep=lambda _: None)
        janitor = merge_janitor.Janitor(client, facts or Facts(), effects or Effects(), dry_run)
        self.assertEqual(janitor.run(NOW), 0, janitor.log)
        self.assertEqual([r for r in inner.requests if r.method != "GET"], [])  # no direct write
        return seen

    def test_step_mode_prints_a_note_as_a_comment_action(self) -> None:
        seen = self.stepped()
        self.assertEqual([(a["action"], a["seq"], a["pr"]) for a in seen], [("comment", 1, 7)])
        self.assertEqual(merge_janitor.parse_note(str(seen[0]["body"])), (PUSHED, RENUMBER))

    def test_step_mode_prints_an_unlabel_and_its_comment(self) -> None:
        self.pulls = [pull(8, HEAD, (READY, "bug"))]
        self.comments = {8: [comment(1, merge_ready.marker_body(marker(OLD)))]}
        self.pushed = {8: "2026-11-01T09:00:00Z"}
        seen = self.stepped(facts=Facts(carries=False))
        self.assertEqual(seen[0], {"action": "unlabel", "seq": 1, "pr": 8, "label": READY, "labels": ["bug"]})
        self.assertEqual((seen[1]["action"], seen[1]["seq"]), ("comment", 2))
        self.assertIn("readiness invalidated by a push", str(seen[1]["body"]))

    def test_step_mode_prints_the_mark_ready_writes(self) -> None:
        # The janitor's mark is merge_ready.apply on its client: in step mode, three actions.
        seen: list[dict[str, object]] = []

        def session(action: dict[str, object]) -> str:
            seen.append(action)
            self.perform(action)
            return merge_step.DONE

        client = merge_step.StepClient(self.fake(), session, sleep=lambda _: None)
        mark = merge_ready.Marker(PUSHED, OLD, "T-0001", {"security": "APPROVE"}, "session-1")
        plan = merge_ready.Plan(7, mark, merge_ready.marker_body(mark), True, True, (READY,))
        merge_ready.apply(client, plan)
        self.assertEqual([a["action"] for a in seen], ["comment", "label", "unlabel"])
        self.assertEqual(seen[1]["labels"], [EJECTED, READY])
        self.assertEqual(seen[2]["labels"], [READY])
        self.assertEqual(merge_ready.parse_marker(str(seen[0]["body"])).prev, OLD)

    def test_step_mode_dry_run_prints_nothing_to_perform(self) -> None:
        self.assertEqual(self.stepped(dry_run=True), [])
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q", tmp], check=True)
            client = merge_step.StepClient(self.fake(), merge_step.AckFile(Path(tmp)), sleep=lambda _: None)
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                merge_janitor.Janitor(client, Facts(), Effects(), dry_run=True).run(NOW)
        self.assertEqual(out.getvalue(), "")

    def test_step_mode_acks_advance_through_the_actions(self) -> None:
        # The real ack flow: each action is printed and pending until `merge_step.py ack` answers it.
        self.pulls = [pull(8, HEAD, (READY,))]
        self.comments = {8: [comment(1, merge_ready.marker_body(marker(OLD)))]}
        self.pushed = {8: "2026-11-01T09:00:00Z"}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(["git", "init", "-q", tmp], check=True)
            acked: list[int] = []

            def sleep(_: float) -> None:  # the session, while the janitor waits
                pending = json.loads((merge_step.step_dir(root) / "pending").read_text())
                self.perform(pending)
                self.assertEqual(merge_step.main(["ack", "--seq", str(pending["seq"])], root), 0)
                acked.append(pending["seq"])

            session = merge_step.AckFile(root, sleep=sleep)
            client = merge_step.StepClient(self.fake(), session, sleep=lambda _: None)
            out = io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
                errors = merge_janitor.Janitor(client, Facts(carries=False), Effects()).run(NOW)
        self.assertEqual(errors, 0)
        self.assertEqual(acked, [1, 2])
        printed = [json.loads(line) for line in out.getvalue().splitlines()]
        self.assertEqual([(a["action"], a["seq"]) for a in printed], [("unlabel", 1), ("comment", 2)])
        self.assertEqual(self.pulls[0]["labels"], [])


class StepFlag(unittest.TestCase):
    def test_step_wraps_the_client_and_a_step_error_exits_2(self) -> None:
        clients: list[object] = []

        def run(self: merge_janitor.Janitor, now: datetime) -> int:
            clients.append(self.client)
            raise merge_step.StepError("step 1 (comment) was not acked within 10 min")

        with (
            mock.patch.object(merge_janitor.gh_client, "from_remote", return_value=FakeGh()),
            mock.patch.object(
                merge_janitor.merge_train, "read_mode", return_value=merge_janitor.merge_train.TRAIN
            ),
            mock.patch.object(merge_janitor.Janitor, "run", run),
            contextlib.redirect_stdout(io.StringIO()),
            contextlib.redirect_stderr(io.StringIO()) as err,
        ):
            self.assertEqual(merge_janitor.main(["--step", "--by", "session-1"]), 2)
        self.assertIsInstance(clients[0], merge_step.StepClient)
        self.assertIn("step mode aborted", err.getvalue())


if __name__ == "__main__":
    unittest.main()
