"""Tests for scripts/merge_step.py, the merge train's step mode (T-3077, decision 0169; stdlib
unittest, run by `make hygiene`)."""

from __future__ import annotations

import contextlib
import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready  # noqa: E402 (the sys.path line above must run first)
import merge_step  # noqa: E402
import merge_train  # noqa: E402
from gh_client import Client, GhError, Request  # noqa: E402
from merge_step import DONE, REFUSED, StepClient, StepError  # noqa: E402
from test_merge_train import GitHub, StageFixture  # noqa: E402

SHA = "a" * 40
VALID: list[dict[str, object]] = [
    {"action": "comment", "pr": 7, "body": "merged"},
    {"action": "label", "pr": 7, "label": "ready-to-merge", "labels": ["bug", "ready-to-merge"]},
    {"action": "unlabel", "pr": 7, "label": "ready-to-merge", "labels": ["bug"]},
    {
        "action": "merge",
        "pr": 7,
        "expectedHeadSha": SHA,
        "title": "T-0001: x (#7)",
        "message": "",
        "method": "squash",
    },
    {
        "action": "create_pull",
        "title": "revert: x",
        "head": "merge-train/revert-1",
        "base": "main",
        "body": "b",
    },
    {"action": "delete_branch", "branch": "agent/harness/T-0001-x"},
    {"action": "dispatch", "workflow": "ci.yml", "ref": "merge-train/current"},
]


class Schema(unittest.TestCase):
    def test_each_action_type_validates(self) -> None:
        self.assertEqual({str(a["action"]) for a in VALID}, set(merge_step.FIELDS))
        for action in VALID:
            with self.subTest(action=action["action"]):
                merge_step.validate(action)
                merge_step.validate({**action, "seq": 3})

    def test_anything_else_is_refused(self) -> None:
        comment, label, unlabel, merge, pull, delete, dispatch = VALID
        bad = [
            {"action": "close", "pr": 7},
            {**comment, "extra": 1},
            {"action": "comment", "pr": 7},
            {**comment, "seq": 0},
            {**comment, "pr": True},
            {**comment, "pr": "7"},
            {**comment, "body": ""},
            {**comment, "body": "x" * 65537},
            {**label, "label": "bug"},
            {**label, "labels": ["bug"]},
            {**label, "labels": ["ready-to-merge", "ready-to-merge"]},
            {**label, "labels": ["bad\nname", "ready-to-merge"]},
            {**unlabel, "labels": ["ready-to-merge"]},
            {**merge, "expectedHeadSha": "HEAD"},
            {**merge, "method": "merge"},
            {**merge, "title": "two\nlines"},
            {**pull, "head": "../x"},
            {**delete, "branch": "main"},
            {**dispatch, "workflow": "../ci.yml"},
        ]
        for action in bad:
            with self.subTest(action=action), self.assertRaises(StepError):
                merge_step.validate(action)

    def test_mark_ready_prints_the_same_schema(self) -> None:
        marker = merge_ready.Marker(SHA, None, "T-0001", {"security": "APPROVE"}, "session-1")
        plan = merge_ready.Plan(7, marker, merge_ready.marker_body(marker), True, ("bug", "ready-to-merge"))
        for action in plan.actions():
            merge_step.validate(action)

    def test_the_footer_is_accepted_once_and_only_at_the_end(self) -> None:
        self.assertTrue(merge_step.same_body("x", "x"))
        self.assertTrue(merge_step.same_body("x" + merge_ready.FOOTER, "x"))
        self.assertTrue(merge_step.same_body(("x" + merge_ready.FOOTER).replace("\n", "\r\n"), "x"))
        self.assertFalse(merge_step.same_body("x" + merge_ready.FOOTER * 2, "x"))
        self.assertFalse(merge_step.same_body(merge_ready.FOOTER + "x", "x"))
        self.assertFalse(merge_step.same_body("x\n\nmore", "x"))


class Hub(Client):
    """A small GitHub: PR 7's labels, comments, merge state, open PRs, branches and dispatches."""

    def __init__(self) -> None:
        super().__init__("o", "r")
        self.labels = ["bug", "ready-to-merge"]
        self.comments_: list[dict[str, object]] = []
        self.merged = ""
        self.opened: list[dict[str, object]] = []
        self.branches = {"agent/harness/T-0001-x"}
        self.dispatches = 0

    def send(self, request: Request) -> object:
        return self.send_cursor(request)[0]

    def send_cursor(self, request: Request) -> tuple[object, str | None]:
        path = request.path.removeprefix("repos/o/r/")
        if path == "pulls/7":
            return {
                "number": 7,
                "labels": [{"name": n} for n in self.labels],
                "head": {"sha": SHA},
                "merged": bool(self.merged),
                "merge_commit_sha": self.merged or None,
            }, None
        if path == "issues/7/comments":
            return self.comments_, None
        if path == f"git/commits/{'b' * 40}":
            return {"message": "T-0001: x (#7)\n\nCo-Authored-By: A <a@b.c>\n"}, None
        if path == "pulls":
            return self.opened, None
        if m := re.fullmatch(r"git/ref/heads/(.+)", path):
            if m[1] in self.branches:
                return {"ref": m[1]}, None
            raise GhError("gh api: HTTP 404 Not Found")
        if path == "actions/runs":
            runs = [{"path": ".github/workflows/ci.yml"}] * self.dispatches
            return {"workflow_runs": runs}, None
        raise GhError(f"hub: no endpoint for {request.method} {path}")

    def perform(self, action: dict[str, object]) -> None:
        """What a session does with the GitHub MCP tools."""
        kind = action["action"]
        if kind in ("label", "unlabel"):
            self.labels = [str(n) for n in action["labels"]]  # type: ignore[union-attr]
        elif kind == "comment":
            self.comments_.append(
                {"body": str(action["body"]) + merge_ready.FOOTER, "author_association": "OWNER"}
            )
        elif kind == "merge":
            self.merged = "b" * 40
        elif kind == "create_pull":
            self.opened.append(
                {
                    "number": 101,
                    "title": action["title"],
                    "body": action["body"],
                    "head": {"ref": action["head"]},
                    "base": {"ref": action["base"]},
                }
            )
        elif kind == "delete_branch":
            self.branches.discard(str(action["branch"]))
        elif kind == "dispatch":
            self.dispatches += 1


class Session:
    """A scripted session: `answer` and `act` decide what it does with each action."""

    def __init__(self, hub: Hub) -> None:
        self.hub = hub
        self.seen: list[dict[str, object]] = []
        self.act: Callable[[dict[str, object]], None] = hub.perform
        self.answer = DONE

    def __call__(self, action: dict[str, object]) -> str:
        self.seen.append(action)
        self.act(action)
        return self.answer


class Writes(unittest.TestCase):
    def setUp(self) -> None:
        self.hub = Hub()
        self.session = Session(self.hub)
        self.client = StepClient(self.hub, self.session, sleep=lambda _: None)

    def test_each_write_is_one_action_read_back(self) -> None:
        self.client.remove_label(7, "ready-to-merge")
        self.client.add_label(7, "merge-train-ejected")
        self.assertEqual(self.hub.labels, ["bug", "merge-train-ejected"])
        self.client.comment(7, "merge train: ejected")
        reply = self.client.merge(7, SHA, title="T-0001: x (#7)", message="Co-Authored-By: A <a@b.c>")
        self.assertEqual(reply, {"sha": "b" * 40, "merged": True})
        pull = self.client.create_pull("revert: x", "merge-train/revert-1", "body")
        self.assertEqual(pull["number"], 101)
        self.client.delete_branch("agent/harness/T-0001-x")
        self.client.dispatch("ci.yml", "merge-train/current")
        self.assertEqual(
            [a["action"] for a in self.session.seen],
            ["unlabel", "label", "comment", "merge", "create_pull", "delete_branch", "dispatch"],
        )
        self.assertEqual([a["seq"] for a in self.session.seen], list(range(1, 8)))
        self.assertEqual(self.session.seen[0]["labels"], ["bug"])
        self.assertEqual(self.session.seen[3]["expectedHeadSha"], SHA)

    def test_a_read_back_that_differs_aborts(self) -> None:
        def wrong_label(action: dict[str, object]) -> None:
            self.hub.labels = ["merge-train-stop"]

        def other_body(action: dict[str, object]) -> None:
            self.hub.comments_.append({"body": "something else", "author_association": "OWNER"})

        def outsider(action: dict[str, object]) -> None:
            self.hub.comments_.append({"body": str(action["body"]), "author_association": "NONE"})

        def other_message(action: dict[str, object]) -> None:
            self.hub.merged = "b" * 40

        writes: list[tuple[Callable[[dict[str, object]], None], Callable[[], object]]] = [
            (wrong_label, lambda: self.client.remove_label(7, "ready-to-merge")),
            (other_body, lambda: self.client.comment(7, "merged")),
            (outsider, lambda: self.client.comment(7, "merged")),
            (other_message, lambda: self.client.merge(7, SHA, title="T-0001: other (#7)")),
        ]
        for act, write in writes:
            with self.subTest(act=act.__name__), self.assertRaises(StepError):
                self.session.act = act
                write()

    def test_an_action_the_session_did_not_perform_aborts(self) -> None:
        self.session.act = lambda _: None
        for write in (
            lambda: self.client.remove_label(7, "ready-to-merge"),
            lambda: self.client.comment(7, "merged"),
            lambda: self.client.merge(7, SHA, title="T-0001: x (#7)"),
            lambda: self.client.delete_branch("agent/harness/T-0001-x"),
            lambda: self.client.dispatch("ci.yml", "merge-train/current"),
        ):
            with self.subTest(write=write), self.assertRaises(StepError):
                write()

    def test_a_refused_write_fails_like_a_direct_one_only_when_nothing_changed(self) -> None:
        self.session.answer = REFUSED
        self.session.act = lambda _: None
        with self.assertRaises(GhError) as caught:
            self.client.delete_branch("agent/harness/T-0001-x")
        self.assertNotIsInstance(caught.exception, StepError)
        self.session.act = self.hub.perform  # "refused", but the branch is gone
        with self.assertRaises(StepError):
            self.client.delete_branch("agent/harness/T-0001-x")

    def test_no_write_bypasses_the_actions(self) -> None:
        for call in (
            lambda: self.client.call("POST", "issues/7/comments", body={"body": "x"}),
            lambda: self.client.create_label("x", "000000", "x"),
            lambda: self.client.dispatch("ci.yml", "main", {"a": "b"}),
        ):
            with self.subTest(call=call), self.assertRaises(StepError):
                call()
        self.assertEqual(self.session.seen, [])
        self.assertEqual(self.client.pull(7)["number"], 7)  # reads go through


class Ack(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)

    def wait(self, on_sleep: Callable[[], None], timeout: float = 60) -> str:
        clock = [0.0]

        def sleep(seconds: float) -> None:
            on_sleep()
            clock[0] += seconds

        session = merge_step.AckFile(self.root, lambda: clock[0], sleep, timeout)
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            answer = session({"action": "comment", "seq": 2, "pr": 7, "body": "x"})
        self.assertEqual(json.loads(out.getvalue()), {"action": "comment", "seq": 2, "pr": 7, "body": "x"})
        return answer

    def ack(self, *argv: str) -> int:
        with contextlib.redirect_stderr(io.StringIO()):
            return merge_step.main(["ack", *argv], self.root)

    def test_the_session_acks_the_pending_action(self) -> None:
        self.assertEqual(self.wait(lambda: self.assertEqual(self.ack("--seq", "2"), 0)), DONE)
        self.assertEqual(self.wait(lambda: self.ack("--seq", "2", "--refused")), REFUSED)
        self.assertFalse((merge_step.step_dir(self.root) / "pending").exists())

    def test_an_ack_for_another_action_is_refused_and_no_ack_aborts(self) -> None:
        self.assertEqual(self.ack("--seq", "2"), 1)  # nothing pending
        with self.assertRaises(StepError):
            self.wait(lambda: self.assertEqual(self.ack("--seq", "3"), 1))  # never acked: times out

    def test_a_forged_ack_file_aborts(self) -> None:
        folder = merge_step.step_dir(self.root)
        with self.assertRaises(StepError):
            self.wait(lambda: (folder / "ack").write_text("1 done"))


class StepSim(GitHub):
    """test_merge_train's simulator plus the reads the step mode's read-back makes: labels on a
    PR, its comments and the branches."""

    def __init__(self, origin: Path, heads: dict[int, str]) -> None:
        super().__init__(origin, heads)
        self.ejected: set[int] = set()
        self.comments_: dict[int, list[dict[str, object]]] = {}
        self.branches = {f"agent/harness/T-000{pr}-x" for pr in heads}

    def answer(self, request: Request) -> object:
        path = request.path.removeprefix("repos/o/r/")
        if request.method == "GET" and (m := re.fullmatch(r"issues/(\d+)/comments", path)):
            return self.comments_.get(int(m[1]), [])
        if request.method == "POST" and (m := re.fullmatch(r"issues/(\d+)/comments", path)):
            assert isinstance(request.body, dict)
            self.writes.append(request)
            body = str(request.body["body"]) + merge_ready.FOOTER
            self.comments_.setdefault(int(m[1]), []).append({"body": body, "author_association": "OWNER"})
            return {}
        if request.method == "POST" and (m := re.fullmatch(r"issues/(\d+)/labels", path)):
            self.writes.append(request)
            self.ejected.add(int(m[1]))
            return {}
        if m := re.fullmatch(r"git/ref/heads/(.+)", path):
            if m[1] in self.branches - set(self.deleted):
                return {"ref": m[1]}
            raise GhError("gh api: HTTP 404 Not Found")
        reply = super().answer(request)
        if m := re.fullmatch(r"pulls/(\d+)", path):
            pr = int(m[1])
            names = [
                *([] if pr in self.unlabelled else ["ready-to-merge"]),
                *(["merge-train-ejected"] if pr in self.ejected else []),
            ]
            assert isinstance(reply, dict)
            reply["labels"] = [{"name": n} for n in names]
        return reply


class StepStage(StageFixture):
    """A whole train in step mode: the session performs each action on the simulator."""

    def setUp(self) -> None:
        super().setUp()
        self.gh = StepSim(self.origin, self.heads)
        self.actions: list[dict[str, object]] = []
        self.refuse: set[str] = set()

        def session(action: dict[str, object]) -> str:
            self.actions.append(action)
            kind, sim = action["action"], self.gh
            if kind in self.refuse:
                return REFUSED
            if kind == "unlabel":
                sim.remove_label(int(str(action["pr"])), str(action["label"]))
            elif kind == "comment":
                sim.comment(int(str(action["pr"])), str(action["body"]))
            elif kind == "merge":
                sim.merge(
                    int(str(action["pr"])),
                    str(action["expectedHeadSha"]),
                    title=str(action["title"]),
                    message=str(action["message"]),
                )
            elif kind == "delete_branch":
                sim.delete_branch(str(action["branch"]))
            return DONE

        client = StepClient(self.gh, session, sleep=lambda _: None)
        self.stage = merge_train.Stage(client, self.clone, self.lock, self.clock, self.clock.sleep)

    def test_a_green_train_merges_through_actions_in_order(self) -> None:
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])
        self.assertEqual(
            [(a["action"], a.get("pr", a.get("branch"))) for a in self.actions],
            [
                ("merge", 1),
                ("unlabel", 1),
                ("comment", 1),
                ("delete_branch", "agent/harness/T-0001-x"),
                ("merge", 2),
                ("unlabel", 2),
                ("comment", 2),
                ("delete_branch", "agent/harness/T-0002-x"),
            ],
        )
        self.assertEqual(
            self.gh.git("rev-parse", "main^{tree}"), self.gh.git("rev-parse", f"{self.train_ref()}^{{tree}}")
        )
        self.assertEqual(self.lock.calls[-1], ("release", None))

    def test_a_refused_branch_deletion_is_logged_as_in_direct_mode(self) -> None:
        self.refuse = {"delete_branch"}
        self.assertEqual(self.run_stage(), 0, self.stage.log)
        self.assertEqual(self.gh.merged, [1, 2])
        self.assertTrue(any("was not deleted" in line for line in self.stage.log), self.stage.log)

    def test_a_skipped_action_aborts_with_nothing_after_it_and_the_lock_released(self) -> None:
        def ack_without_acting(action: dict[str, object]) -> str:
            self.actions.append(action)
            return DONE  # acked, never performed

        self.stage.client = StepClient(self.gh, ack_without_acting, sleep=lambda _: None)
        with self.assertRaises(StepError):
            self.run_stage()
        self.assertEqual([a["action"] for a in self.actions], ["merge"])
        self.assertEqual(self.gh.merged, [])
        self.assertEqual(self.lock.calls[-1], ("release", None))


if __name__ == "__main__":
    unittest.main()
