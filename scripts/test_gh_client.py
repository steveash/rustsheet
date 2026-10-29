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


class Refusals(unittest.TestCase):
    def test_no_caller_text_reaches_another_path(self) -> None:
        fake = FakeGh()
        refused: list[Callable[[], object]] = [
            lambda: fake.pull("7/../../x"),
            lambda: fake.pull(0),
            lambda: fake.pull("٧"),
            lambda: fake.permission("a/b"),
            lambda: fake.add_label(7, "../x"),
            lambda: fake.check_runs("HEAD"),
            lambda: fake.commit(SHA.upper()),
            lambda: fake.workflow_runs(branch="-x"),
            lambda: fake.dispatch("../ci.yml", "main"),
            lambda: fake.merge(7, SHA, title="a\nb"),
            lambda: fake.merge(7, SHA, title="t", method="fast-forward"),
            lambda: fake.comment(7, ""),
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
