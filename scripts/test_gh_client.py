"""Tests for scripts/gh_client.py: the fake and `gh api` build the same requests (SPEC-0027)."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from collections.abc import Callable
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gh_client  # noqa: E402 (the sys.path line above must run first)
from gh_client import Client, FakeGh, GhApi, GhError, Reply  # noqa: E402

SHA = "0123456789abcdef0123456789abcdef01234567"
PAGE = "per_page=100&page=1"

# Recorded fixtures: each operation, the (method, url, body) it sends and the reply it parses.
FIXTURES: list[tuple[str, Callable[[Client], object], str, str, object, object]] = [
    ("pull", lambda c: c.pull(7), "GET", "repos/o/r/pulls/7", None, {"state": "open"}),
    ("pulls", lambda c: c.pulls(), "GET", f"repos/o/r/pulls?state=open&{PAGE}", None, []),
    (
        "closed_labelled",
        lambda c: c.closed_labelled("ready-to-merge"),
        "GET",
        f"repos/o/r/issues?state=closed&labels=ready-to-merge&{PAGE}",
        None,
        [],
    ),
    ("pull_files", lambda c: c.pull_files(7), "GET", f"repos/o/r/pulls/7/files?{PAGE}", None, []),
    ("comments", lambda c: c.comments(7), "GET", f"repos/o/r/issues/7/comments?{PAGE}", None, []),
    ("reviews", lambda c: c.reviews(7), "GET", f"repos/o/r/pulls/7/reviews?{PAGE}", None, []),
    (
        "permission",
        lambda c: c.permission("octo-cat"),
        "GET",
        "repos/o/r/collaborators/octo-cat/permission",
        None,
        {"permission": "write"},
    ),
    ("comment", lambda c: c.comment(7, "hi"), "POST", "repos/o/r/issues/7/comments", {"body": "hi"}, {"id": 1}),
    (
        "add_label",
        lambda c: c.add_label(7, "ready-to-merge"),
        "POST",
        "repos/o/r/issues/7/labels",
        {"labels": ["ready-to-merge"]},
        [],
    ),
    ("label", lambda c: c.label("merge train"), "GET", "repos/o/r/labels/merge%20train", None, {"name": "x"}),
    (
        "create_label",
        lambda c: c.create_label("ready-to-merge", "0e8a16", "Ready"),
        "POST",
        "repos/o/r/labels",
        {"name": "ready-to-merge", "color": "0e8a16", "description": "Ready"},
        {"name": "ready-to-merge"},
    ),
    (
        "remove_label",
        lambda c: c.remove_label(7, "merge train"),
        "DELETE",
        "repos/o/r/issues/7/labels/merge%20train",
        None,
        [],
    ),
    (
        "check_runs",
        lambda c: c.check_runs(SHA),
        "GET",
        f"repos/o/r/commits/{SHA}/check-runs?{PAGE}",
        None,
        {"check_runs": []},
    ),
    (
        "workflow_runs",
        lambda c: c.workflow_runs(branch="merge-train/current", head_sha=SHA),
        "GET",
        f"repos/o/r/actions/runs?branch=merge-train%2Fcurrent&head_sha={SHA}&{PAGE}",
        None,
        {"workflow_runs": []},
    ),
    (
        "workflow_runs by event and time",
        lambda c: c.workflow_runs(event="pull_request", created=">=2026-10-28T00:00:00Z"),
        "GET",
        f"repos/o/r/actions/runs?event=pull_request&created=%3E%3D2026-10-28T00%3A00%3A00Z&{PAGE}",
        None,
        {"workflow_runs": []},
    ),
    ("run_jobs", lambda c: c.run_jobs(42), "GET", f"repos/o/r/actions/runs/42/jobs?{PAGE}", None, {"jobs": []}),
    (
        "updated_pulls",
        lambda c: c.updated_pulls("2026-10-28T00:00:00Z"),
        "GET",
        f"repos/o/r/pulls?state=all&sort=updated&direction=desc&{PAGE}",
        None,
        [],
    ),
    ("commit", lambda c: c.commit(SHA), "GET", f"repos/o/r/git/commits/{SHA}", None, {"sha": SHA}),
    (
        "merge",
        lambda c: c.merge(7, SHA, title="T-0001: x (#7)", message="body"),
        "PUT",
        "repos/o/r/pulls/7/merge",
        {"merge_method": "squash", "sha": SHA, "commit_title": "T-0001: x (#7)", "commit_message": "body"},
        {"merged": True},
    ),
    (
        "create_pull",
        lambda c: c.create_pull("revert: x", "merge-train/revert-1", "body"),
        "POST",
        "repos/o/r/pulls",
        {"title": "revert: x", "head": "merge-train/revert-1", "base": "main", "body": "body"},
        {"number": 8},
    ),
    (
        "delete_branch",
        lambda c: c.delete_branch("agent/harness/T-0001-x"),
        "DELETE",
        "repos/o/r/git/refs/heads/agent/harness/T-0001-x",
        None,
        None,
    ),
    (
        "dispatch",
        lambda c: c.dispatch("ci.yml", "main", {"suite": "smoke"}),
        "POST",
        "repos/o/r/actions/workflows/ci.yml/dispatches",
        {"ref": "main", "inputs": {"suite": "smoke"}},
        None,
    ),
]


class SameRequests(unittest.TestCase):
    def test_the_fake_and_gh_api_send_each_fixture(self) -> None:
        for name, operation, method, url, body, reply in FIXTURES:
            with self.subTest(name):
                fake = FakeGh("o", "r", {(method, url): Reply(reply)})
                operation(fake)
                [request] = fake.requests
                self.assertEqual((request.method, request.url(), request.body), (method, url, body))

                calls: list[tuple[list[str], str]] = []

                def run(argv: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    calls.append((argv, str(kwargs["input"])))
                    return subprocess.CompletedProcess(argv, 0, json.dumps(reply), "")

                with mock.patch("gh_client.subprocess.run", side_effect=run):
                    operation(GhApi("o", "r"))
                [(argv, stdin)] = calls
                accept = ["-H", "Accept: application/vnd.github+json"]
                tail = ["--input", "-"] if body is not None else []
                self.assertEqual(argv, ["gh", "api", "--method", method, *accept, url, *tail])
                self.assertEqual(json.loads(stdin) if body is not None else None, body)


class Pages(unittest.TestCase):
    def test_lists_page_until_a_short_page_and_stay_bounded(self) -> None:
        full = list(range(gh_client.PER_PAGE))
        fake = FakeGh(
            replies={
                ("GET", "repos/o/r/pulls?state=all&per_page=100&page=1"): Reply(full),
                ("GET", "repos/o/r/pulls?state=all&per_page=100&page=2"): Reply([1]),
            }
        )
        self.assertEqual(len(fake.pulls("all")), gh_client.PER_PAGE + 1)

        class Endless(Client):
            def send(self, request: gh_client.Request) -> object:
                return full

        with self.assertRaises(GhError):
            Endless("o", "r").pulls()


class Cursor(unittest.TestCase):
    """The activity endpoint ignores `page=`: its read follows the `after=` cursor of the Link header."""

    URL = "repos/o/r/activity?ref=refs%2Fheads%2Fmerge-lock&per_page=100"

    def test_the_fake_follows_the_cursor_back_past_since(self) -> None:
        fake = FakeGh(replies={
            ("GET", self.URL): Reply([{"timestamp": "2026-10-29T10:00:00Z"}], after="Y3Vy"),
            ("GET", f"{self.URL}&after=Y3Vy"): Reply([{"timestamp": "2026-10-27T10:00:00Z"}], after="bmV4"),
        })
        self.assertEqual(len(fake.activity("merge-lock", "2026-10-28T00:00:00Z")), 2)
        self.assertEqual(len(fake.requests), 2)

    def test_gh_api_reads_the_cursor_from_the_link_header(self) -> None:
        link = '<https://api.github.com/repositories/1/activity?ref=x&after=Y3Vy%3D>; rel="next"'
        outputs = [f"HTTP/2.0 200 OK\r\nLink: {link}\r\n\r\n[]", "HTTP/2.0 200 OK\r\n\r\n[]"]
        calls: list[list[str]] = []

        def run(argv: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
            calls.append(argv)
            return subprocess.CompletedProcess(argv, 0, outputs[len(calls) - 1], "")

        with mock.patch("gh_client.subprocess.run", side_effect=run):
            self.assertEqual(GhApi("o", "r").activity("merge-lock", "2026-10-28T00:00:00Z"), [])
        # An empty page ends the read even with a cursor; the cursor goes back on the repos/ URL.
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][:3], ["gh", "api", "--include"])
        self.assertEqual(calls[0][-1], self.URL)

    def test_a_malformed_cursor_is_refused(self) -> None:
        out = 'HTTP/2.0 200 OK\nLink: <https://x/activity?after=a/b>; rel="next"\n\n[]'
        with mock.patch("gh_client.subprocess.run", return_value=subprocess.CompletedProcess([], 0, out, "")):
            with self.assertRaises(GhError):
                GhApi("o", "r").activity("merge-lock", "2026-10-28T00:00:00Z")


class Refusals(unittest.TestCase):
    def test_no_caller_text_reaches_another_path(self) -> None:
        fake = FakeGh()
        refused: list[Callable[[], object]] = [
            lambda: fake.pull("7/../../x"),
            lambda: fake.pull(0),
            lambda: fake.pull("٧"),
            lambda: fake.permission("a/b"),
            lambda: fake.add_label(7, "../x"),
            lambda: fake.label("../x"),
            lambda: fake.create_label("x", "#00ff00", "d"),
            lambda: fake.create_label("x", "00ff00", ""),
            lambda: fake.check_runs("HEAD"),
            lambda: fake.commit(SHA.upper()),
            lambda: fake.workflow_runs(branch="-x"),
            lambda: fake.workflow_runs(created="2026-10-28&x=1"),
            lambda: fake.workflow_runs(event="push&x=1"),
            lambda: fake.run_jobs("1/../x"),
            lambda: fake.activity("../x", "2026"),
            lambda: fake.dispatch("../ci.yml", "main"),
            lambda: fake.merge(7, SHA, title="a\nb"),
            lambda: fake.merge(7, SHA, title="t", method="fast-forward"),
            lambda: fake.comment(7, ""),
            lambda: fake.delete_branch("main"),
            lambda: fake.delete_branch("a/../../../pulls"),
            lambda: fake.delete_branch("a//b"),
            lambda: fake.delete_branch("a/.x"),
            lambda: fake.delete_branch("a.lock"),
            lambda: fake.create_pull("t", "../x", "body"),
            lambda: fake.create_pull("a\nb", "x", "body"),
            lambda: fake.create_pull("t", "x", ""),
            lambda: FakeGh("o/x", "r"),
            lambda: FakeGh("..", "r"),
            lambda: fake.pull("1" * 5000),
        ]
        for call in refused:
            with self.assertRaises(GhError):
                call()
        self.assertEqual(fake.requests, [])


class Failures(unittest.TestCase):
    def test_gh_failures_raise_and_pr_finished_reads_them_as_unknown(self) -> None:
        def failing(*_: object, **__: object) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess([], 1, "", "gh: Not Found (HTTP 404)\n")

        with mock.patch("gh_client.subprocess.run", side_effect=failing):
            with self.assertRaisesRegex(GhError, "HTTP 404"):
                GhApi("o", "r").pull(7)
            self.assertIsNone(GhApi("o", "r").pr_finished(7))
            # A missing label is an answer, not a failure; any other failed read raises.
            self.assertIsNone(GhApi("o", "r").label("ready-to-merge"))
        self.assertRaises(GhError, FakeGh().label, "ready-to-merge")
        with mock.patch("gh_client.subprocess.run", side_effect=FileNotFoundError("gh")):
            self.assertIsNone(GhApi("o", "r").pr_finished(7))
        bad = subprocess.CompletedProcess([], 0, "<html>", "")
        with mock.patch("gh_client.subprocess.run", return_value=bad), self.assertRaises(GhError):
            GhApi("o", "r").pull(7)

    def test_a_remote_names_the_repository(self) -> None:
        self.assertEqual(gh_client.owner_repo("https://github.com/o/r.git"), ("o", "r"))
        self.assertEqual(gh_client.owner_repo("git@github.com:o/r"), ("o", "r"))
        self.assertIsNone(gh_client.owner_repo("https://example.com/o/r"))


if __name__ == "__main__":
    unittest.main()
