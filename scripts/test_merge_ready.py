"""Tests for scripts/merge_ready.py: the marker, reviewer applicability, the "green" rule and V1-V6
(SPEC-0027 section 2 and its Test plan), against the fake GitHub client and a temporary remote."""

from __future__ import annotations

import contextlib
import io
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready
from gh_client import FakeGh, Reply
from merge_ready import IGNORE, KEEP, UNLABEL, Marker, MarkerError

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


class Verify(unittest.TestCase):
    """V1-V6 on PR #7, whose head adds a task file on a branch cut from `main`."""

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
        self.pull: dict[str, object] = {
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
        self.pull["base"]["repo"]["owner"]["type"] = "Organization"  # type: ignore[index]
        self.assertEqual(self.failures(), [("V1", IGNORE)])
        self.permissions["collab"] = Reply({"permission": "write"})
        self.assertEqual(self.failures(), [])

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
        self.pull["head"]["sha"] = "c" * 40  # type: ignore[index]
        self.assertEqual(self.failures(), [("V2", UNLABEL)])
        # GitHub's head agrees with the marker but origin's refs/pull/7/head moved.
        self.pull["head"]["sha"] = self.head  # type: ignore[index]
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
        self.pull["head"]["sha"] = self.head  # type: ignore[index]
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
        self.pull["head"]["sha"] = self.head  # type: ignore[index]
        self.comments = [self.comment(1, marker(self.head).render())]
        self.checks = green_checks()
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        self.comments = [
            self.comment(1, marker(self.head, reviewers={**APPROVED, "excel-fidelity": "APPROVE"}).render())
        ]
        self.assertEqual(self.failures(), [])

    def test_pr_state_and_the_task_file(self) -> None:
        self.pull["draft"] = True
        self.pull["base"]["ref"] = "release"  # type: ignore[index]
        self.pull["head"]["repo"] = None  # type: ignore[index]
        self.assertEqual(self.failures(), [("V5", UNLABEL)] * 3)
        self.pull["draft"] = False
        self.pull["base"]["ref"] = "main"  # type: ignore[index]
        self.pull["head"]["repo"] = {"full_name": "o/r"}  # type: ignore[index]
        for status, outcome in (("blocked", "## Outcome\n"), ("done", "")):
            with self.subTest(status=status):
                self.head = self.push_head(TASK.format(status=status, outcome=outcome))
                self.pull["head"]["sha"] = self.head  # type: ignore[index]
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


if __name__ == "__main__":
    unittest.main()
