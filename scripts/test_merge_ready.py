"""Tests for scripts/merge_ready.py: the marker, reviewer applicability, the "green" rule and V1-V6
(SPEC-0027 section 2 and its Test plan), against the fake GitHub client and a temporary remote."""

from __future__ import annotations

import contextlib
import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gh_client
import merge_ready
from gh_client import FakeGh, Reply
from merge_ready import APPROVE, IGNORE, KEEP, UNLABEL, Marker, MarkerError

ROOT = Path(__file__).resolve().parent.parent
PAGE = "per_page=100&page=1"
SUITE = 900
APPROVED = {"security": "APPROVE", "architecture": "APPROVE", "quality": "APPROVE"}
TASK = """---
id: T-0001
status: {status}
---

## Goal
x
{outcome}"""


def run(*args: str, cwd: Path) -> str:
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


def marker(head: str, **changes: object) -> Marker:
    fields: dict[str, object] = {
        "head": head,
        "prev": None,
        "task": "T-0001",
        "reviewers": APPROVED,
        "marked_by": "session-1",
    }
    fields.update(changes)
    return Marker(**fields)  # type: ignore[arg-type]


def check_run(
    ident: int, name: str, conclusion: str = "success", app: int = 15368, suite: int = SUITE
) -> dict[str, object]:
    return {
        "id": ident,
        "name": name,
        "status": "completed",
        "conclusion": conclusion,
        "app": {"id": app},
        "check_suite": {"id": suite},
    }


def green_checks(**conclusions: str) -> list[dict[str, object]]:
    names = (*merge_ready.REQUIRED_JOBS, *merge_ready.HEAVY_JOBS)
    return [check_run(i + 1, name, conclusions.get(name, "success")) for i, name in enumerate(names)]


def ci_run(
    conclusion: str = "success", status: str = "completed", ident: int = 1, event: str = "pull_request"
) -> dict[str, object]:
    return {
        "id": ident,
        "path": merge_ready.CI_WORKFLOW,
        "event": event,
        "status": status,
        "conclusion": conclusion,
        "check_suite_id": SUITE,
    }


class MarkerFormat(unittest.TestCase):
    HEAD = "a" * 40

    def test_round_trip(self) -> None:
        mark = marker(self.HEAD, prev="b" * 40, reviewers={**APPROVED, "excel-fidelity": "APPROVE"})
        self.assertEqual(
            merge_ready.parse_marker("Ready.\r\n\r\n" + mark.render().replace("\n", "\r\n")), mark
        )
        bare = marker(self.HEAD, task=None)
        self.assertIn("excel-fidelity=n/a", bare.render())
        self.assertEqual(
            merge_ready.parse_marker(bare.render()).reviewers, {**APPROVED, "excel-fidelity": "n/a"}
        )

    def test_one_session_footer_after_the_block_is_accepted(self) -> None:
        # T-3164: the session's write path appends this footer to every comment it posts.
        mark = marker(self.HEAD, reviewers={**APPROVED, "excel-fidelity": "n/a"})
        body = merge_ready.marker_body(mark) + merge_ready.FOOTER
        self.assertEqual(merge_ready.parse_marker(body), mark)
        self.assertEqual(merge_ready.parse_marker(body.replace("\n", "\r\n")), mark)
        # T-3177: a trailing newline after the footer is tolerated too.
        self.assertEqual(merge_ready.parse_marker(body + "\n"), mark)
        self.assertEqual(merge_ready.parse_marker((body + "\n").replace("\n", "\r\n")), mark)
        with self.assertRaises(MarkerError):
            merge_ready.parse_marker(body + "\n\n")
        with self.assertRaises(MarkerError):
            merge_ready.parse_marker(body + "\nx")

    def test_anything_else_after_the_block_is_rejected(self) -> None:
        good = merge_ready.marker_body(marker(self.HEAD))
        footer = merge_ready.FOOTER
        bad = [
            good + footer + footer,
            good + footer + "\ntrailing text",
            good + "\ntrailing text" + footer,
            good + footer.replace("Claude Code", "Claude C0de"),
            good + "\n\n---",
        ]
        for body in bad:
            with self.subTest(body=body[-60:]), self.assertRaises(MarkerError):
                merge_ready.parse_marker(body)

    def test_bad_fields_are_rejected(self) -> None:
        good = marker(self.HEAD).render()
        bad = [
            good.replace(self.HEAD, "A" * 40),
            good.replace(self.HEAD, self.HEAD[:39]),
            good.replace("prev: none", "prev: HEAD"),
            good.replace("security=APPROVE", "sekurity=APPROVE"),
            good.replace("quality=APPROVE", "quality=APPROVE quality=APPROVE"),
            good.replace("quality=APPROVE", "quality=LGTM"),
            good.replace("task: T-0001", "task: T-1"),
            good.replace("marked-by: session-1", "marked-by: a b"),
            good.replace("head:", "Head:"),
            good.replace("\n-->", ""),
            good + "\ntrailing text",
            "no marker at all",
        ]
        for body in bad:
            with self.subTest(body=body[-60:]), self.assertRaises(MarkerError):
                merge_ready.parse_marker(body)


class Applicability(unittest.TestCase):
    def test_excel_fidelity_exactly_for_the_skill_paths(self) -> None:
        always = merge_ready.ALWAYS_REVIEWERS
        self.assertEqual(merge_ready.applicable_reviewers(["scripts/x.py", "docs/a.md"]), always)
        self.assertEqual(merge_ready.applicable_reviewers(["crates/rustsheet-engine-extra/x.rs"]), always)
        self.assertEqual(merge_ready.applicable_reviewers(None), merge_ready.REVIEWERS)
        for root in merge_ready.EXCEL_FIDELITY_PATHS:
            with self.subTest(root):
                self.assertEqual(merge_ready.applicable_reviewers([f"{root}/x"]), merge_ready.REVIEWERS)

    def test_the_list_matches_the_review_skill(self) -> None:
        skill = (ROOT / ".claude/skills/review/SKILL.md").read_text()
        always = tuple(re.findall(r"`reviewer-([a-z-]+)` \(always\)", skill))
        self.assertEqual(always, merge_ready.ALWAYS_REVIEWERS)
        clause = re.search(r"`reviewer-excel-fidelity` \(if the diff touches ([^)]*)\)", skill)
        assert clause is not None, "the review skill no longer names excel-fidelity's paths"
        paths = tuple(p.strip() for p in re.split(r",|\bor\b", " ".join(clause[1].split())) if p.strip())
        self.assertEqual(paths, merge_ready.EXCEL_FIDELITY_PATHS)

    def test_the_job_names_match_ci(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        names = set(re.findall(r"^    name: (.+)$", workflow, re.M))
        for job in (*merge_ready.REQUIRED_JOBS, *merge_ready.HEAVY_JOBS):
            self.assertIn(job, names)


class Green(unittest.TestCase):
    def state(
        self, runs: list[dict[str, object]], checks: list[dict[str, object]], docs_only: bool | None = False
    ) -> str:
        return merge_ready.ci_state(runs, checks, docs_only)[0]

    def test_the_green_rule(self) -> None:
        eval_job, python_job = merge_ready.HEAVY_JOBS
        skipped = green_checks(**{eval_job: "skipped", python_job: "skipped"})
        self.assertEqual(self.state([ci_run()], green_checks()), merge_ready.GREEN)
        # Skipped heavy jobs only with run_heavy=false.
        self.assertEqual(self.state([ci_run()], skipped, docs_only=True), merge_ready.GREEN)
        self.assertEqual(self.state([ci_run()], skipped, docs_only=False), merge_ready.RED)
        self.assertEqual(self.state([ci_run()], skipped, docs_only=None), merge_ready.RED)
        hygiene = merge_ready.REQUIRED_JOBS[1]
        self.assertEqual(self.state([ci_run()], green_checks(**{hygiene: "skipped"}), True), merge_ready.RED)
        # A cancelled run is never green, whatever its jobs say.
        self.assertEqual(self.state([ci_run("cancelled")], green_checks()), merge_ready.RED)
        self.assertEqual(
            self.state([ci_run(), ci_run("cancelled", ident=2)], green_checks()), merge_ready.RED
        )
        self.assertEqual(self.state([ci_run("", "in_progress")], green_checks()), merge_ready.PENDING)
        self.assertEqual(self.state([ci_run(event="push")], green_checks()), merge_ready.PENDING)

    def test_check_runs_from_other_apps_or_suites_are_ignored(self) -> None:
        name = merge_ready.REQUIRED_JOBS[0]
        forged = [*green_checks(**{name: "failure"}), check_run(99, name, app=1)]
        self.assertEqual(self.state([ci_run()], forged), merge_ready.RED)
        stray = [c for c in green_checks() if c["name"] != name] + [check_run(99, name, suite=1)]
        self.assertEqual(self.state([ci_run()], stray), merge_ready.RED)
        # The newest check run of a name counts (a re-run of a failed job).
        rerun = [*green_checks(**{name: "failure"}), check_run(99, name)]
        self.assertEqual(self.state([ci_run()], rerun), merge_ready.GREEN)
        pending = [*green_checks(), {**check_run(99, name), "status": "queued", "conclusion": None}]
        self.assertEqual(self.state([ci_run()], pending), merge_ready.PENDING)


class PrFixture(unittest.TestCase):
    """PR #7, whose head adds a task file on a branch cut from `main`, marked, green and approved."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        base = Path(self.tmp.name)
        origin, self.clone = base / "origin.git", base / "clone"
        run("git", "init", "-q", "--bare", "-b", "main", str(origin), cwd=base)
        run("git", "clone", "-q", str(origin), str(self.clone), cwd=base)
        self.git("config", "user.name", "t")
        self.git("config", "user.email", "t@invalid")
        self.git("checkout", "-q", "-b", "main")
        self.write("README", "x")
        self.base_sha = self.commit("base")
        self.git("push", "-q", "origin", "main")
        self.head = self.push_head(TASK.format(status="done", outcome="## Outcome\ndone\n"))
        self.comments: list[dict[str, object]] = [self.comment(1, marker(self.head).render())]
        self.reviews: list[dict[str, object]] = []
        self.runs = [ci_run()]
        self.checks = green_checks()
        self.pull: dict[str, Any] = {
            "labels": [{"name": merge_ready.READY_LABEL}],
            "draft": False,
            "head": {"sha": self.head, "ref": "agent/harness/T-0001-x", "repo": {"full_name": "o/r"}},
            "base": {
                "sha": self.base_sha,
                "ref": "main",
                "repo": {"full_name": "o/r", "owner": {"type": "User"}},
            },
        }
        self.permissions: dict[str, Reply] = {}

    def git(self, *args: str) -> str:
        return run("git", *args, cwd=self.clone)

    def write(self, path: str, text: str) -> None:
        file = self.clone / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD")

    def push_head(self, task: str, path: str = "docs/tasks/M2/T-0001-x.md") -> str:
        self.git("checkout", "-q", "-B", "pr", self.base_sha)
        self.write(path, task)
        sha = self.commit("pr head")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        return sha

    @staticmethod
    def comment(
        ident: int,
        body: str,
        login: str = "owner",
        association: str = "OWNER",
        at: str = "2026-10-28T10:00:00Z",
    ) -> dict[str, object]:
        return {
            "id": ident,
            "body": body,
            "user": {"login": login},
            "author_association": association,
            "updated_at": at,
        }

    def fake(self) -> FakeGh:
        replies = {
            ("GET", "repos/o/r/pulls/7"): Reply(self.pull),
            ("GET", f"repos/o/r/issues/7/comments?{PAGE}"): Reply(self.comments),
            ("GET", f"repos/o/r/pulls/7/reviews?{PAGE}"): Reply(self.reviews),
            ("GET", f"repos/o/r/actions/runs?head_sha={self.head}&{PAGE}"): Reply(
                {"workflow_runs": self.runs}
            ),
            ("GET", f"repos/o/r/commits/{self.head}/check-runs?{PAGE}"): Reply({"check_runs": self.checks}),
        }
        for login in ("owner", "outsider", "collab"):
            reply = self.permissions.get(login, Reply(error="HTTP 403 (the session proxy)"))
            replies[("GET", f"repos/o/r/collaborators/{login}/permission")] = reply
        return FakeGh("o", "r", replies)

    def verify(self) -> tuple[merge_ready.Verdict, FakeGh]:
        fake = self.fake()
        # The classifier logs a count of changed files to stderr.
        with contextlib.redirect_stderr(io.StringIO()):
            return merge_ready.verify(fake, 7, merge_ready.Repo(self.clone)), fake

    def failures(self) -> list[tuple[str, str]]:
        verdict, fake = self.verify()
        # verify only reads.
        self.assertEqual({r.method for r in fake.requests}, {"GET"})
        return [(f.check, f.action) for f in verdict.failures]



class Verify(PrFixture):
    """V1-V6 on PR #7."""

    def test_a_marked_green_approved_pr_is_ready(self) -> None:
        verdict, _ = self.verify()
        self.assertTrue(verdict.ready, verdict.failures)
        self.assertEqual((verdict.comment_id, verdict.updated_at), (1, "2026-10-28T10:00:00Z"))
        out, err = io.StringIO(), io.StringIO()
        with (
            mock.patch.object(merge_ready.gh_client, "from_remote", return_value=self.fake()),
            contextlib.redirect_stdout(out),
            contextlib.redirect_stderr(err),
        ):
            self.assertEqual(merge_ready.main(["verify", "--pr", "7"], root=self.clone), 0)
            self.assertEqual(merge_ready.main(["verify", "--pr", "x/../7"], root=self.clone), 2)
        self.assertIn("#7 ready", out.getvalue())
        self.assertIn("refused PR number", err.getvalue())

    def test_no_label_or_no_marker_skips_silently(self) -> None:
        self.comments = []
        self.assertEqual(self.failures(), [("V1", IGNORE)])
        self.pull["labels"] = []
        self.assertEqual(self.failures(), [("label", IGNORE)])

    def test_an_outsiders_marker_is_ignored(self) -> None:
        # Newer, for another head, by an outsider: it neither counts nor invalidates.
        stale = marker("b" * 40).render()
        self.comments.append(self.comment(2, stale, "outsider", "CONTRIBUTOR", "2026-10-28T11:00:00Z"))
        self.assertEqual(self.failures(), [])
        self.comments = self.comments[1:]
        self.assertEqual(self.failures(), [("V1", IGNORE)])
        # In CI the permission API decides, whatever the association says.
        self.permissions["outsider"] = Reply({"permission": "read"})
        self.comments[0]["author_association"] = "OWNER"
        self.assertEqual(self.failures(), [("V1", IGNORE)])
        # On an organization's repository the association alone is not enough.
        self.comments = [self.comment(1, marker(self.head).render(), "collab", "COLLABORATOR")]
        self.assertEqual(self.failures(), [])
        self.pull["base"]["repo"]["owner"]["type"] = "Organization"
        self.assertEqual(self.failures(), [("V1", IGNORE)])
        self.permissions["collab"] = Reply({"permission": "write"})
        self.assertEqual(self.failures(), [])
        # Only the proxy's refusal falls back to the association; another failed read is an error.
        self.permissions["collab"] = Reply(error="gh api GET x: HTTP 502")
        with self.assertRaises(merge_ready.GhError):
            self.verify()
        # An item without a valid login is never a writer.
        self.comments = [self.comment(1, marker(self.head).render(), "", "OWNER")]
        self.assertEqual(self.failures(), [("V1", IGNORE)])

    def test_the_newest_writer_marker_wins_and_an_edit_rekeys(self) -> None:
        self.comments.append(self.comment(2, marker("b" * 40).render(), at="2026-10-28T11:00:00Z"))
        self.assertEqual(self.failures(), [("V2", UNLABEL)])
        # Editing the older comment makes it the newest marker, with its new updated_at.
        self.comments[0]["updated_at"] = "2026-10-28T12:00:00Z"
        verdict, _ = self.verify()
        self.assertTrue(verdict.ready)
        self.assertEqual((verdict.comment_id, verdict.updated_at), (1, "2026-10-28T12:00:00Z"))
        self.comments.append(self.comment(3, "x " + merge_ready.MARKER_OPEN, at="2026-10-28T13:00:00Z"))
        self.assertEqual(self.failures(), [("V1", UNLABEL)])

    def test_a_stale_head_invalidates(self) -> None:
        self.pull["head"]["sha"] = "c" * 40
        self.assertEqual(self.failures(), [("V2", UNLABEL)])
        # GitHub's head agrees with the marker but origin's refs/pull/7/head moved.
        self.pull["head"]["sha"] = self.head
        self.git("commit", "-q", "--allow-empty", "-m", "later")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.assertEqual(self.failures(), [("V2", UNLABEL)])

    def test_pending_ci_keeps_the_label_and_red_ci_unlabels(self) -> None:
        self.runs = [ci_run("", "in_progress")]
        self.assertEqual(self.failures(), [("V3", KEEP)])
        self.runs = [ci_run("failure")]
        self.assertEqual(self.failures(), [("V3", UNLABEL)])

    def test_skipped_heavy_jobs_count_only_for_a_docs_only_diff(self) -> None:
        skipped = dict.fromkeys(merge_ready.HEAVY_JOBS, "skipped")
        self.checks = green_checks(**skipped)
        self.assertEqual(self.failures(), [])
        self.head = self.push_head(TASK.format(status="done", outcome="## Outcome\n"), "scripts/x.py")
        self.write("docs/tasks/M2/T-0001-x.md", TASK.format(status="done", outcome="## Outcome\n"))
        self.head = self.commit("code and task")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.comments = [self.comment(1, marker(self.head).render())]
        self.assertEqual(self.failures(), [("V3", UNLABEL)])

    def test_verdicts_must_cover_the_applicable_reviewers(self) -> None:
        self.comments = [
            self.comment(1, marker(self.head, reviewers={**APPROVED, "quality": "REQUEST_CHANGES"}).render())
        ]
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # An engine change needs excel-fidelity too.
        self.head = self.push_head(TASK.format(status="done", outcome="## Outcome\n"))
        self.write("crates/rustsheet-engine/src/x.rs", "x")
        self.head = self.commit("engine")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.comments = [self.comment(1, marker(self.head).render())]
        self.checks = green_checks()
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        self.comments = [
            self.comment(1, marker(self.head, reviewers={**APPROVED, "excel-fidelity": "APPROVE"}).render())
        ]
        self.assertEqual(self.failures(), [])

    def test_pr_state_and_the_task_file(self) -> None:
        self.pull["draft"] = True
        self.pull["base"]["ref"] = "release"
        self.pull["head"]["repo"] = None
        self.assertEqual(self.failures(), [("V5", UNLABEL)] * 3)
        self.pull["draft"] = False
        self.pull["base"]["ref"] = "main"
        self.pull["head"]["repo"] = {"full_name": "o/r"}
        for status, outcome in (("blocked", "## Outcome\n"), ("done", "")):
            with self.subTest(status=status):
                self.head = self.push_head(TASK.format(status=status, outcome=outcome))
                self.pull["head"]["sha"] = self.head
                self.comments = [self.comment(1, marker(self.head).render())]
                self.assertEqual(self.failures(), [("V5", UNLABEL)])

    def test_task_only_prs_skip_the_task_check(self) -> None:
        self.head = self.push_head("not a task file", "docs/tasks/M2/T-0002-y.md")
        self.pull["head"] = {"sha": self.head, "ref": "tasks/refill", "repo": {"full_name": "o/r"}}
        self.comments = [self.comment(1, marker(self.head, task=None).render())]
        self.assertEqual(self.failures(), [])
        # A marker naming a task on a branch without one does not match it.
        self.comments = [self.comment(1, marker(self.head).render())]
        self.assertEqual(self.failures(), [("V5", UNLABEL)])

    def test_changes_requested_counts_only_from_a_writer_after_the_marker(self) -> None:
        review = {"state": "CHANGES_REQUESTED", "submitted_at": "2026-10-28T11:00:00Z"}
        self.reviews = [{**review, "user": {"login": "outsider"}, "author_association": "NONE"}]
        self.assertEqual(self.failures(), [])
        self.reviews.append(
            {
                **review,
                "submitted_at": "2026-10-28T09:00:00Z",
                "user": {"login": "collab"},
                "author_association": "COLLABORATOR",
            }
        )
        self.assertEqual(self.failures(), [])
        self.reviews.append({**review, "user": {"login": "collab"}, "author_association": "COLLABORATOR"})
        self.assertEqual(self.failures(), [("V6", UNLABEL)])


class Mark(PrFixture):
    """`mark` on the same PR #7, unmarked and unlabelled, its head checked out locally."""

    REVIEW = "security=APPROVE architecture=APPROVE quality=APPROVE"

    def setUp(self) -> None:
        super().setUp()
        self.comments.clear()
        self.pull["labels"] = [{"name": "bug"}]

    def plan(self, review: str | None = REVIEW, carry: bool = False, by: str = "s-1") -> merge_ready.Plan:
        with contextlib.redirect_stderr(io.StringIO()):
            return merge_ready.plan_mark(self.fake(), 7, merge_ready.Repo(self.clone), review, carry, by)

    def refused(self, reason: str, **kwargs: Any) -> None:
        with self.assertRaisesRegex(merge_ready.Refusal, reason):
            self.plan(**kwargs)

    def perform(self, action: dict[str, object]) -> None:
        """What a session does with one step-mode action through the GitHub MCP tools."""
        if action["action"] == merge_ready.STEP_COMMENT:
            self.comments.append(self.comment(len(self.comments) + 1, str(action["body"])))
        else:
            label = merge_ready.READY_LABEL if action["action"] == merge_ready.STEP_LABEL else merge_ready.EJECTED_LABEL
            self.assertEqual(action["label"], label)
            self.pull["labels"] = [{"name": name} for name in action["labels"]]  # type: ignore[attr-defined]

    def mark_main(self, *args: str, fake: FakeGh | None = None) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with (
            mock.patch.object(merge_ready.gh_client, "from_remote", return_value=fake or self.fake()),
            contextlib.redirect_stdout(out),
            contextlib.redirect_stderr(err),
        ):
            code = merge_ready.main(["mark", "--pr", "7", "--by", "s-1", *args], root=self.clone)
        return code, out.getvalue(), err.getvalue()

    def test_step_mode_round_trip_passes_verify(self) -> None:
        code, out, _ = self.mark_main("--review", self.REVIEW, "--step")
        self.assertEqual(code, 0)
        actions = [json.loads(line) for line in out.splitlines()]
        self.assertEqual([a["action"] for a in actions], ["comment", "label"])
        self.assertEqual(actions[1]["labels"], ["bug", merge_ready.READY_LABEL])
        for action in actions:
            self.perform(action)
        verdict, _ = self.verify()
        self.assertTrue(verdict.ready, verdict.failures)
        assert verdict.marker is not None
        reviewers = {**APPROVED, "excel-fidelity": "n/a"}
        self.assertEqual(verdict.marker, marker(self.head, reviewers=reviewers, marked_by="s-1"))
        # Never a second marker for the same head: nothing left to do.
        code, out, _ = self.mark_main("--review", self.REVIEW, "--step")
        self.assertEqual((code, out.strip()), (0, f"#7 is already marked at {self.head}"))
        # A lost label is put back without a second comment.
        self.pull["labels"] = []
        self.assertEqual([a["action"] for a in self.plan().actions()], ["label"])

    def test_marking_again_drops_the_ejected_label(self) -> None:
        # SPEC-0027 section 6: `merge-train-ejected` goes once the PR is marked again (T-3155).
        self.pull["labels"] = [{"name": "bug"}, {"name": merge_ready.EJECTED_LABEL}]
        code, out, _ = self.mark_main("--review", self.REVIEW, "--step")
        self.assertEqual(code, 0)
        actions = [json.loads(line) for line in out.splitlines()]
        self.assertEqual([a["action"] for a in actions], ["comment", "label", "unlabel"])
        final = ["bug", merge_ready.READY_LABEL]
        self.assertEqual([a.get("labels") for a in actions[1:]], [final, final])
        for action in actions:
            self.perform(action)
        self.assertEqual([label["name"] for label in self.pull["labels"]], final)
        verdict, _ = self.verify()
        self.assertTrue(verdict.ready, verdict.failures)
        # Already marked and labelled at this head, the ejected label alone is removed.
        self.pull["labels"] = [{"name": merge_ready.READY_LABEL}, {"name": merge_ready.EJECTED_LABEL}]
        self.assertEqual([a["action"] for a in self.plan().actions()], ["unlabel"])

    def test_direct_mode_creates_labels_comments_then_labels_and_reads_back(self) -> None:
        test = self

        class Writing(FakeGh):
            def send(self, request: gh_client.Request) -> object:
                if request.method == "POST" and request.path.endswith("/comments"):
                    test.perform({"action": "comment", "body": (request.body or {})["body"]})
                elif request.method == "POST" and request.path.endswith("/issues/7/labels"):
                    test.pull["labels"].append({"name": merge_ready.READY_LABEL})
                elif request.method == "DELETE" and request.path.endswith("/issues/7/labels/merge-train-ejected"):
                    test.pull["labels"].remove({"name": merge_ready.EJECTED_LABEL})
                return super().send(request)

        self.pull["labels"].append({"name": merge_ready.EJECTED_LABEL})
        fake = Writing("o", "r", self.fake().replies)
        missing = Reply(error="gh: Not Found (HTTP 404)")
        fake.replies[("GET", "repos/o/r/labels/ready-to-merge")] = Reply({"name": "ready-to-merge"})
        fake.replies[("GET", "repos/o/r/labels/merge-train-ejected")] = missing
        fake.replies[("GET", "repos/o/r/labels/merge-train-stop")] = missing
        for write in ("labels", "issues/7/comments", "issues/7/labels"):
            fake.replies[("POST", f"repos/o/r/{write}")] = Reply({})
        fake.replies[("DELETE", "repos/o/r/issues/7/labels/merge-train-ejected")] = Reply({})
        code, out, _ = self.mark_main("--review", self.REVIEW, fake=fake)
        self.assertEqual(code, 0)
        self.assertIn("#7 ready", out)
        writes = [(r.method, r.path, (r.body or {}).get("name")) for r in fake.requests if r.method != "GET"]
        self.assertEqual(
            writes,
            [
                ("POST", "repos/o/r/labels", "merge-train-ejected"),
                ("POST", "repos/o/r/labels", "merge-train-stop"),
                ("POST", "repos/o/r/issues/7/comments", None),
                ("POST", "repos/o/r/issues/7/labels", None),
                ("DELETE", "repos/o/r/issues/7/labels/merge-train-ejected", None),
            ],
        )
        self.assertEqual([label["name"] for label in self.pull["labels"]], ["bug", merge_ready.READY_LABEL])

    def test_each_refusal(self) -> None:
        self.refused("--by", by="")
        self.refused("REVIEW", review=None)
        self.refused("REVIEW", carry=True)
        self.refused("unknown or repeated reviewer", review="security=APPROVE lint=APPROVE")
        self.refused("no APPROVE from quality", review="security=APPROVE architecture=APPROVE")
        self.refused("no APPROVE from quality", review=self.REVIEW.replace("quality=APPROVE", "quality=n/a"))
        self.runs = [ci_run("", "in_progress")]
        self.refused("CI is pending")
        self.runs = [ci_run("failure")]
        self.refused("CI is red")
        self.runs = [ci_run()]
        for key, value, reason in (
            ("draft", True, "draft"),
            ("draft", None, "draft"),
            ("base", {**self.pull["base"], "ref": "release"}, "base is not main"),
            ("head", {**self.pull["head"], "repo": {"full_name": "fork/r"}}, "not in this repository"),
            ("head", {**self.pull["head"], "sha": "c" * 40}, "differ"),
        ):
            with self.subTest(reason):
                saved = self.pull[key]
                self.pull[key] = value
                self.refused(reason)
                self.pull[key] = saved
        # The local checkout is not the PR head.
        self.git("checkout", "-q", "main")
        self.refused("differ")
        self.git("checkout", "-q", "pr")
        # The task file on the head is not done.
        self.head = self.push_head(TASK.format(status="in-progress", outcome="## Outcome\n"))
        self.pull["head"]["sha"] = self.head
        self.refused("T-0001 is not status: done")
        self.head = self.push_head(TASK.format(status="done", outcome="## Outcome\n"))
        self.pull["head"]["sha"] = self.head
        self.plan()  # the reasons above were the only ones
        # A label name gh_client would refuse never reaches a step-mode action.
        self.pull["labels"] = [{"name": "../x"}]
        with self.assertRaises(merge_ready.GhError):
            self.plan()

    def test_an_engine_change_needs_excel_fidelity(self) -> None:
        self.write("crates/rustsheet-engine/src/x.rs", "x")
        self.head = self.commit("engine")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.refused("no APPROVE from excel-fidelity")
        plan = self.plan(review=self.REVIEW + " excel-fidelity=APPROVE")
        self.assertEqual(plan.marker.reviewers["excel-fidelity"], APPROVE)

    def merge_main(self) -> str:
        """Advances `main` by one commit and merges it into the PR head, cleanly."""
        old = self.head
        self.git("checkout", "-q", "main")
        self.write("other.txt", "main moved")
        self.commit("main moves")
        self.git("push", "-q", "origin", "main")
        self.git("checkout", "-q", "pr")
        self.git("merge", "-q", "--no-edit", "main")
        self.head = self.git("rev-parse", "HEAD")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.comments.append(self.comment(1, marker(old, reviewers={**APPROVED, "excel-fidelity": "n/a"}).render()))
        return old

    def test_carry_accepts_a_clean_merge_of_main(self) -> None:
        self.refused("earlier marker", review=None, carry=True)
        old = self.merge_main()
        plan = self.plan(review=None, carry=True)
        self.assertEqual((plan.marker.prev, plan.marker.head), (old, self.head))
        self.assertEqual(plan.marker.reviewers, {**APPROVED, "excel-fidelity": "n/a"})
        self.perform(plan.actions()[0])
        # A re-run after the comment landed but not the label adds only the label.
        again = self.plan(review=None, carry=True)
        self.assertEqual([a["action"] for a in again.actions()], ["label"])
        self.perform(again.actions()[0])
        verdict, _ = self.verify()
        self.assertTrue(verdict.ready, verdict.failures)

    def test_carry_refuses_an_edited_merge_or_another_commit(self) -> None:
        self.merge_main()
        self.write("scripts/x.py", "edited in the merge")
        self.git("add", "-A")
        self.git("commit", "-q", "--amend", "--no-edit")
        self.head = self.git("rev-parse", "HEAD")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.refused("not clean", review=None, carry=True)
        # A plain commit on the marked head is not a merge.
        self.git("reset", "-q", "--hard", "HEAD^1")
        self.write("scripts/x.py", "a later change")
        self.head = self.commit("later")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.refused("not a merge commit", review=None, carry=True)
        # A merge of a commit that is not on main.
        self.git("reset", "-q", "--hard", "HEAD^1")
        side = self.git("commit-tree", "-p", self.base_sha, "-m", "side", f"{self.base_sha}^{{tree}}")
        self.git("merge", "-q", "--no-edit", side)
        self.head = self.git("rev-parse", "HEAD")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.refused("not on main", review=None, carry=True)


if __name__ == "__main__":
    unittest.main()
