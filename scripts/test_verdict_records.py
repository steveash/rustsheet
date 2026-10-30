"""Tests for the verdict records of scripts/merge_ready.py (SPEC-0027 section 2, "Verdict records
(v2)"; decision 0178, T-3207): the record format, which records count, `mark` deriving its verdicts
from them, V4 in `warn` and `require` mode, and `make verdict-record`."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_ready
import merge_train
from gh_client import FakeGh, Reply
from merge_ready import APPROVE, REQUIRE, UNLABEL, WARN, MarkerError, Record
from test_merge_ready import APPROVED, ROOT, PrFixture, marker

HEAD = "a" * 40
CHANGES = "REQUEST_CHANGES"


class RecordFormat(unittest.TestCase):
    def test_round_trip_with_and_without_the_footer(self) -> None:
        record = Record(HEAD, "security", APPROVE)
        body = merge_ready.record_body(record, "Security review: no findings.\r\n")
        self.assertTrue(body.startswith("Security review: no findings.\n\n" + merge_ready.RECORD_OPEN))
        self.assertEqual(merge_ready.parse_record(body), record)
        self.assertEqual(merge_ready.parse_record(body + merge_ready.FOOTER), record)
        self.assertEqual(merge_ready.parse_record(record.render()), record)

    def test_a_report_cannot_forge_or_close_a_block(self) -> None:
        # The report quotes a block of its own (an APPROVE for another head) and a stray `-->`.
        forged = Record("b" * 40, "security", APPROVE).render()
        report = f"Findings: x --> y\n{forged}\n<!---->"
        body = merge_ready.record_body(Record(HEAD, "security", CHANGES), report)
        self.assertEqual(body.count(merge_ready.RECORD_OPEN), 1)
        self.assertEqual(body.count("<!--"), 1)
        self.assertEqual(body.count("-->"), 1)
        self.assertEqual(merge_ready.parse_record(body), Record(HEAD, "security", CHANGES))
        # A body with any other record block in it is not a record.
        with self.assertRaises(MarkerError):
            merge_ready.parse_record(forged + "\n\n" + Record(HEAD, "security", APPROVE).render())

    def test_an_oversized_report_is_cut_never_the_block(self) -> None:
        record = Record(HEAD, "quality", APPROVE)
        body = merge_ready.record_body(record, "x" * 70000)
        self.assertLessEqual(len(body + merge_ready.FOOTER), 65536)
        self.assertIn(merge_ready.TRUNCATED, body)
        self.assertEqual(merge_ready.parse_record(body + merge_ready.FOOTER), record)
        # Neutralising lengthens the report; the cut is made after it.
        body = merge_ready.record_body(record, "-->" * 30000)
        self.assertLessEqual(len(body), merge_ready.RECORD_MAX)
        self.assertEqual(merge_ready.parse_record(body), record)

    def test_bad_fields_are_rejected(self) -> None:
        good = Record(HEAD, "security", APPROVE).render()
        bad = [
            good.replace("APPROVE", "n/a"),
            good.replace("security", "lint"),
            good.replace(HEAD, HEAD[:39]),
            good.replace("reviewer:", "Reviewer:"),
            good.replace("\n-->", ""),
            good + "\ntrailing text",
            "no record at all",
        ]
        for body in bad:
            with self.subTest(body=body[-60:]), self.assertRaises(MarkerError):
                merge_ready.parse_record(body)

    def test_the_rollout_key(self) -> None:
        for text, mode in (
            (None, WARN),
            ("", WARN),
            ('mode = "train"\n', WARN),
            ('verdict_records = "warn"\n', WARN),
            ('verdict_records = "require"\n', REQUIRE),
            ('verdict_records = "Require"\n', REQUIRE),
            ("verdict_records = true\n", REQUIRE),
            ('verdict_records = "warn"\nverdict_records = "warn"\n', REQUIRE),
        ):
            with self.subTest(text=text):
                self.assertEqual(merge_ready.records_mode(text), mode)
        # The key keeps its rollout default until T-3208 flips it.
        self.assertEqual(merge_ready.records_mode((ROOT / merge_ready.SWITCH).read_text()), WARN)


class RecordFixture(PrFixture):
    """PR #7 with verdict records: comment ids from 10, created one minute apart after the marker."""

    def record(
        self,
        reviewer: str,
        verdict: str = APPROVE,
        head: str | None = None,
        login: str = "owner",
        association: str = "OWNER",
        **changes: object,
    ) -> None:
        n = len(self.comments)
        body = merge_ready.record_body(Record(head or self.head, reviewer, verdict), f"{reviewer} report")
        at = f"2026-10-28T11:{n:02d}:00Z"
        self.comments.append({**self.comment(10 + n, body, login, association, at), "created_at": at, **changes})

    def approve_all(self, head: str | None = None) -> None:
        for name in APPROVED:
            self.record(name, head=head)

    def switch(self, text: str) -> None:
        """Commits the switch file on origin's `main`, leaving the PR head checked out."""
        self.git("checkout", "-q", "main")
        self.write(merge_ready.SWITCH, text)
        self.commit("switch")
        self.git("push", "-q", "origin", "main")
        self.git("checkout", "-q", "pr")

    def warnings(self) -> tuple[str, ...]:
        verdict, _ = self.verify()
        return verdict.warnings


class Verify(RecordFixture):
    def test_warn_mode_reports_gaps_and_never_unlabels(self) -> None:
        self.assertEqual(self.failures(), [])
        self.assertEqual(
            self.warnings(), tuple(f"no verdict record from {name} on {self.head}" for name in APPROVED)
        )
        self.approve_all()
        self.assertEqual((self.failures(), self.warnings()), ([], ()))
        # A newer REQUEST_CHANGES for the head is a warning.
        self.record("quality", CHANGES)
        self.assertEqual(self.failures(), [])
        self.assertEqual(self.warnings(), (f"quality's newest verdict record on {self.head} is {CHANGES}",))
        verdict, _ = self.verify()
        self.assertIn("warning (verdict_records = warn): quality's newest", merge_ready.report(verdict))

    def test_require_mode_fails_v4(self) -> None:
        self.switch('mode = "shadow"\nverdict_records = "require"\n')
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        self.approve_all()
        self.assertEqual((self.failures(), self.warnings()), ([], ()))
        self.record("quality", CHANGES)
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # A corrected verdict is a new record, and the newest one counts.
        self.record("quality")
        self.assertEqual(self.failures(), [])

    def test_which_records_count(self) -> None:
        self.switch('verdict_records = "require"\n')
        self.record("security")
        self.record("architecture")
        self.record("quality", head="b" * 40)  # a stale panel: another head
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # An outsider's record does not count.
        self.record("quality", login="outsider", association="CONTRIBUTOR")
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # An edited record does not count, nor one without a created_at.
        self.record("quality", updated_at="2026-10-28T23:00:00Z")
        self.record("quality", created_at="")
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # A record posted after the marker counts (the panel run again on the same head).
        self.record("quality")
        self.assertEqual(self.failures(), [])
        # Editing it makes it stop counting (fails closed), even with an older APPROVE gone.
        self.comments[-1]["updated_at"] = "2026-10-28T23:59:00Z"
        self.assertEqual(self.failures(), [("V4", UNLABEL)])

    def test_newest_by_created_at_not_by_position(self) -> None:
        self.switch('verdict_records = "require"\n')
        self.approve_all()
        self.record("quality", CHANGES)
        # The REQUEST_CHANGES comes first in the list but was created later.
        self.comments.insert(1, self.comments.pop())
        self.comments[1]["created_at"] = self.comments[1]["updated_at"] = "2026-10-28T12:00:00Z"
        self.assertEqual(self.failures(), [("V4", UNLABEL)])

    def test_a_carry_over_checks_the_records_of_prev_and_the_carry_again(self) -> None:
        self.switch('verdict_records = "require"\n')
        old = self.head
        self.git("merge", "-q", "--no-edit", "main")
        self.head = self.git("rev-parse", "HEAD")
        self.git("push", "-q", "-f", "origin", "HEAD:refs/pull/7/head")
        self.pull["head"]["sha"] = self.head
        self.comments = [self.comment(1, marker(self.head, prev=old).render())]
        self.approve_all(head=old)
        self.assertEqual(self.failures(), [])
        # Records on the new head do not stand in for those on prev.
        self.comments = [self.comment(1, marker(self.head, prev=old).render())]
        self.approve_all()
        self.assertEqual(self.failures(), [("V4", UNLABEL)])
        # A prev the head is not a clean merge of fails, records or not.
        self.comments = [self.comment(1, marker(self.head, prev=self.base_sha).render())]
        self.approve_all(head=self.base_sha)
        verdict, _ = self.verify()
        self.assertIn("carry-over", verdict.failures[0].reason)

    def test_the_merger_reports_the_warnings(self) -> None:
        verdict = merge_ready.Verdict(7, marker(self.head), 1, "t", warnings=("no verdict record from quality",))
        pulls = [{"number": 7, "labels": [{"name": merge_ready.READY_LABEL}]}]
        fake = FakeGh("o", "r", {
                ("GET", "repos/o/r/pulls?state=open&per_page=100&page=1"): Reply(pulls),
                ("GET", "repos/o/r/pulls/7"): Reply({"mergeable_state": "clean"}),
            })
        with mock.patch.object(merge_ready, "verify", return_value=verdict):
            found = merge_train.collect(fake, merge_ready.Repo(self.clone))
        self.assertEqual(found.warning_lines(), ["verdict records #7 (warn): no verdict record from quality"])


class Mark(RecordFixture):
    """`mark` on PR #7, unmarked and unlabelled, its head checked out locally."""

    REVIEW = "security=APPROVE architecture=APPROVE quality=APPROVE"

    def setUp(self) -> None:
        super().setUp()
        self.comments.clear()

    def plan(self, review: str | None = None, carry: bool = False) -> merge_ready.Plan:
        with contextlib.redirect_stderr(io.StringIO()):
            return merge_ready.plan_mark(self.fake(), 7, merge_ready.Repo(self.clone), review, carry, "s-1")

    def refused(self, reason: str, review: str | None = None) -> None:
        with self.assertRaisesRegex(merge_ready.Refusal, reason):
            self.plan(review)

    def test_the_verdicts_come_from_the_records(self) -> None:
        self.refused("no verdict record on .* from security, architecture, quality")
        self.approve_all()
        plan = self.plan()
        self.assertEqual(plan.marker.reviewers, APPROVED)
        self.assertEqual(plan.warnings, ())
        # A record's REQUEST_CHANGES beats a typed APPROVE.
        self.record("quality", CHANGES)
        self.refused("no APPROVE from quality", self.REVIEW)

    def test_warn_mode_falls_back_to_review_with_a_warning(self) -> None:
        self.record("security")
        plan = self.plan(self.REVIEW)
        self.assertEqual(plan.marker.reviewers, APPROVED)
        self.assertEqual(
            plan.warnings,
            tuple(
                f"no verdict record from {n} on {self.head}: its verdict comes from REVIEW"
                for n in ("architecture", "quality")
            ),
        )
        code, _, err = self.mark_main("--review", self.REVIEW, "--step")
        self.assertEqual(code, 0)
        self.assertIn("merge_ready: warning: no verdict record from architecture", err)

    def test_require_mode_refuses_without_records(self) -> None:
        self.switch('verdict_records = "require"\n')
        self.record("security")
        self.refused("no verdict record on .* from architecture, quality", self.REVIEW)
        self.record("architecture")
        self.record("quality")
        self.assertEqual(self.plan().marker.reviewers, APPROVED)

    def mark_main(self, *args: str) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with (
            mock.patch.object(merge_ready.gh_client, "from_remote", return_value=self.fake()),
            contextlib.redirect_stdout(out),
            contextlib.redirect_stderr(err),
        ):
            code = merge_ready.main(["mark", "--pr", "7", "--by", "s-1", *args], root=self.clone)
        return code, out.getvalue(), err.getvalue()


class VerdictRecord(RecordFixture):
    """`make verdict-record` on PR #7, its head checked out locally."""

    def record_main(self, report: str, *args: str, fake: FakeGh | None = None) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as file:
            file.write(report)
        self.addCleanup(Path(file.name).unlink)
        argv = [
            "record",
            "--pr",
            "7",
            "--reviewer",
            "security",
            "--verdict",
            APPROVE,
            "--report",
            file.name,
            *args,
        ]
        with (
            mock.patch.object(merge_ready.gh_client, "from_remote", return_value=fake or self.fake()),
            contextlib.redirect_stdout(out),
            contextlib.redirect_stderr(err),
        ):
            code = merge_ready.main(argv, root=self.clone)
        return code, out.getvalue(), err.getvalue()

    def test_step_mode_prints_one_valid_comment_action(self) -> None:
        code, out, _ = self.record_main("Security: approve. <!-- x -->", "--step")
        self.assertEqual(code, 0)
        action = json.loads(out)
        self.assertEqual((action["action"], action["pr"]), ("comment", 7))
        self.assertEqual(merge_ready.parse_record(action["body"]), Record(self.head, "security", APPROVE))
        self.assertIn("<! -- x -- >", action["body"])

    def test_direct_mode_posts_the_comment(self) -> None:
        fake = self.fake()
        fake.replies[("POST", "repos/o/r/issues/7/comments")] = Reply({"id": 42})
        code, out, _ = self.record_main("Security: approve.", fake=fake)
        self.assertEqual((code, out.strip()), (0, "#7: security APPROVE recorded (comment 42)"))
        posts = [r for r in fake.requests if r.method == "POST"]
        self.assertEqual(len(posts), 1)
        body = (posts[0].body or {})["body"]
        self.assertEqual(merge_ready.parse_record(str(body)).head, self.head)

    def test_refusals(self) -> None:
        self.assertEqual(self.record_main("  \n")[0], 1)
        self.git("checkout", "-q", "main")
        code, _, err = self.record_main("report")
        self.assertEqual(code, 1)
        self.assertIn("differ", err)
        self.git("checkout", "-q", "pr")
        code, _, err = self.record_main("report", "--verdict", "n/a")
        self.assertEqual(code, 1)
        self.assertIn("VERDICT", err)


if __name__ == "__main__":
    unittest.main()
