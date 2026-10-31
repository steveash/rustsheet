"""Tests for merge_train_stats.py (`make merge-train-stats`, T-3076): the agreement rules of SPEC-0027
section 4 on scripted PR histories, the lock and train groupings, and a whole report from the fake client."""

from __future__ import annotations

import contextlib
import io
import sys
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_lock  # noqa: E402 (the sys.path line above must run first)
import merge_train_stats as stats  # noqa: E402
from gh_client import FakeGh, GhError, Request  # noqa: E402
from merge_train_shadow import PREDICTION_HEADER, SHADOW_BRANCH  # noqa: E402
from merge_train_stats import AGREE, CHANGE, CLEAN, CONFLICT, DISAGREE, RENUMBER, UNSETTLED, Later  # noqa: E402

T0 = datetime(2026, 10, 29, 0, 0, tzinfo=UTC)


def sha(n: int) -> str:
    return f"{n:040x}"


def at(hours: float) -> datetime:
    return T0 + timedelta(hours=hours)


def stamp(hours: float) -> str:
    return stats.iso(at(hours))


def later(
    steps: list[tuple[int, str, float]],
    merged: tuple[int, float] | None = None,
    runs: dict[int, list[str]] | None = None,
) -> Later:
    """A PR history: commits (sha n, kind, hour), the merge (head n, hour) and run conclusions."""
    history = Later(
        at(merged[1]) if merged else None,
        sha(merged[0]) if merged else "",
        [(sha(n), kind, at(h)) for n, kind, h in steps],
    )
    history.conclusions = {sha(n): c for n, c in (runs or {}).items()}
    return history


class Agreement(unittest.TestCase):
    """SPEC-0027 section 4, "When a shadow prediction agrees", one scripted history per rule."""

    def verdict(self, outcome: str, history: Later) -> str | None:
        return stats.score(outcome, sha(1), at(1), history)

    def test_merge(self) -> None:
        same = later([(1, CHANGE, 0)], (1, 2), {1: ["success"]})
        carried = later([(1, CHANGE, 0), (2, CLEAN, 2)], (2, 3), {2: ["failure", "success"]})
        red = later([(1, CHANGE, 0)], (1, 2), {1: ["failure"]})
        changed = later([(1, CHANGE, 0), (2, CHANGE, 2)], (2, 3), {2: ["success"]})
        late = later([(1, CHANGE, 0)], (1, 30), {1: ["success"]})
        self.assertEqual(self.verdict("merge", same), AGREE)
        self.assertEqual(self.verdict("merge", carried), AGREE)
        self.assertEqual(self.verdict("merge", red), DISAGREE)
        self.assertEqual(self.verdict("merge", changed), DISAGREE)
        self.assertEqual(self.verdict("merge", late), UNSETTLED)

    def test_ci_red(self) -> None:
        went_red = later([(1, CHANGE, 0), (2, CLEAN, 2)], None, {2: ["failure"]})
        fixed = later([(1, CHANGE, 0), (2, CHANGE, 2)], (2, 3), {2: ["success"]})
        unchanged = later([(1, CHANGE, 0)], (1, 2), {1: ["success"]})
        self.assertEqual(self.verdict("eject:ci-red", went_red), AGREE)
        self.assertEqual(self.verdict("eject:ci-red", fixed), AGREE)
        self.assertEqual(self.verdict("eject:ci-red", unchanged), DISAGREE)
        self.assertEqual(self.verdict("eject:ci-red", later([(1, CHANGE, 0)])), UNSETTLED)

    def test_conflict_and_id_collision(self) -> None:
        resolved = later([(1, CHANGE, 0), (2, CONFLICT, 2)], (2, 3), {2: ["success"]})
        renumbered = later([(1, CHANGE, 0), (2, CLEAN, 2), (3, RENUMBER, 3)], (3, 4))
        unchanged = later([(1, CHANGE, 0)], (1, 2), {1: ["success"]})
        for outcome in ("eject:conflict", "eject:id-collision"):
            self.assertEqual(self.verdict(outcome, resolved), AGREE)
            self.assertEqual(self.verdict(outcome, renumbered), AGREE)
            self.assertEqual(self.verdict(outcome, unchanged), DISAGREE)

    def test_unscored_outcomes_and_rewritten_branches(self) -> None:
        unchanged = later([(1, CHANGE, 0)], (1, 2), {1: ["success"]})
        for outcome in ("defer", "no result", "eject:invalidated"):
            self.assertIsNone(self.verdict(outcome, unchanged))
        # The predicted head is gone from the PR (a force push): a non-merge change.
        rewritten = later([(9, CHANGE, 2)], (9, 3), {9: ["success"]})
        self.assertEqual(self.verdict("merge", rewritten), DISAGREE)
        self.assertEqual(self.verdict("eject:ci-red", rewritten), AGREE)


class Parsing(unittest.TestCase):
    def test_a_prediction_commit(self) -> None:
        message = "\n".join(
            [
                PREDICTION_HEADER,
                "",
                f"shadow-input: {sha(9)} #1 {sha(1)} #2 {sha(2)}",
                "#1 merge",
                "#2 eject:ci-red",
                "#3 defer",
                "unlabel: #1, #2",
                "tree-check: match",
            ]
        )
        found = stats.parse_prediction(sha(5), at(0), message)
        assert found is not None
        self.assertEqual(found.heads, {1: sha(1), 2: sha(2)})
        self.assertEqual(found.outcomes, {1: "merge", 2: "eject:ci-red", 3: "defer"})
        self.assertEqual(found.tree_check, "match")
        self.assertTrue(found.approximate)
        self.assertIsNone(stats.parse_prediction(sha(5), at(0), "merge train on main x"))

    def test_percentile_is_nearest_rank(self) -> None:
        self.assertEqual(stats.percentile([5, 1, 3, 2, 4, 6, 7, 8, 9, 10], 0.9), 9)
        self.assertIsNone(stats.percentile([], 0.5))


class Groupings(unittest.TestCase):
    def test_lock_holds_and_their_waits(self) -> None:
        free = merge_lock.Lock("", None)
        queued = merge_lock.Lock(
            "", "a", "7", waiters=(merge_lock.Waiter("b", "8", int(at(0.5).timestamp()), 0),)
        )
        b = merge_lock.Lock("", "b", "8")
        holds = stats.holds_of([(at(0), free), (at(0.25), queued), (at(1), b), (at(2), free)], at(3))
        self.assertEqual(
            [(h.pr, h.start, h.end, h.wait) for h in holds],
            [("7", at(0.25), at(1), 0.0), ("8", at(1), at(2), 1800.0)],
        )

    def test_trains_group_a_batch_and_its_bisection(self) -> None:
        def run(n: int, conclusion: str) -> stats.Run:
            return stats.Run(n, SHADOW_BRANCH, sha(n), "push", conclusion, at(n), at(n), at(n))

        tagged = [
            (run(1, "failure"), "m0", frozenset({1, 2, 3})),
            (run(2, "success"), "m0", frozenset({1})),
            (run(3, "success"), "m0", frozenset({4})),
            (run(4, "success"), "m1", frozenset({4})),
        ]
        self.assertEqual([len(t.runs) for t in stats.trains_of(tagged)], [2, 1, 1])


class Routed(FakeGh):
    """The fake client keyed by path: a page past the first is empty, so fixtures need no query."""

    def __init__(self, replies: dict[str, object]) -> None:
        super().__init__()
        self.routes = replies

    def send_cursor(self, request: Request) -> tuple[object, str | None]:
        self.requests.append(request)
        if dict(request.query).get("page", "1") != "1":
            empty: object = {"workflow_runs": [], "jobs": []} if "actions/" in request.path else []
            return empty, None
        path = request.path.removeprefix(self.base + "/")
        if path == "activity":
            path += "@" + dict(request.query)["ref"].removeprefix("refs/heads/")
        if dict(request.query).get("head_sha"):
            path += "@" + dict(request.query)["head_sha"]
        if path not in self.routes:
            raise GhError(f"fake: no route for {path}")
        return self.routes[path], None


class NoGit:
    def clean_merge(self, pr: int, prev: str, head: str) -> bool:
        return True

    def renumbers(self, pr: int, prev: str, head: str) -> bool:
        return False


class Report(unittest.TestCase):
    def test_a_whole_report_from_the_fake_client(self) -> None:
        def ci_run(
            n: int,
            branch: str,
            head: str,
            hours: float,
            conclusion: str = "success",
            event: str = "pull_request",
        ) -> dict[str, object]:
            return {
                "id": n,
                "path": ".github/workflows/ci.yml",
                "head_branch": branch,
                "head_sha": head,
                "event": event,
                "conclusion": conclusion,
                "created_at": stamp(hours),
                "run_started_at": stamp(hours),
                "updated_at": stamp(hours + 0.1),
            }

        prediction = "\n".join(
            [
                PREDICTION_HEADER,
                "",
                f"shadow-input: {sha(9)} #1 {sha(1)}",
                "#1 merge",
                "unlabel: #1",
                "tree-check: match",
            ]
        )
        lock_held = "merge-lock\n\nversion: 2\nholder: s1\npr: 1\nrenewed: 0\n"
        lock_free = "merge-lock\n\nversion: 2\nstate: free\nfreed: 0\n"
        window = [
            ci_run(10, "agent/x", sha(1), 1.0),
            ci_run(11, "agent/x", sha(1), 2.2),
            ci_run(12, SHADOW_BRANCH, sha(20), 1.5, event="push"),
        ]
        routes: dict[str, object] = {
            f"activity@{SHADOW_BRANCH}": [{"timestamp": stamp(1.6), "after": sha(21)}],
            "activity@merge-lock": [
                {"timestamp": stamp(3), "after": sha(31)},
                {"timestamp": stamp(2), "after": sha(30)},
            ],
            f"git/commits/{sha(21)}": {"message": prediction},
            f"git/commits/{sha(20)}": {"message": f"merge train on main {sha(9)}\n\n#1 {sha(1)}"},
            f"git/commits/{sha(30)}": {"message": lock_held},
            f"git/commits/{sha(31)}": {"message": lock_free},
            "actions/runs": {"workflow_runs": window},
            f"actions/runs@{sha(1)}": {"workflow_runs": window[:2]},
            "actions/runs/10/jobs": {
                "jobs": [{"created_at": stamp(1), "started_at": stamp(1.01), "completed_at": stamp(1.1)}]
            },
            "actions/runs/11/jobs": {"jobs": []},
            "actions/runs/12/jobs": {"jobs": []},
            "pulls": [
                {
                    "number": 1,
                    "merged_at": stamp(1.6),
                    "updated_at": stamp(1.6),
                    "base": {"ref": "main"},
                    "head": {"sha": sha(1), "ref": "agent/x"},
                    "labels": [],
                }
            ],
            "pulls/1": {"merged": True, "merged_at": stamp(1.6), "head": {"sha": sha(1)}},
            "pulls/1/commits": [
                {"sha": sha(1), "parents": [], "commit": {"committer": {"date": stamp(0.5)}}}
            ],
        }
        report = stats.Stats(Routed(routes), NoGit(), at(4), at(0)).report()
        expected = [
            "merged PRs: 1; merges per hour mean 0.25, peak hour 1",
            "latency under 3/h: n=1 p50 30.0 min, p90 30.0 min",
            "merge-stage CI runs per merged PR: 1.00 (1 runs under the lock)",
            "lock: 1 holds; wait p90 0.0 min; held 25% of the time",
            "stranded PRs: none",
            f"{SHADOW_BRANCH}: 1 trains, red-train rate 0% (0/1), bisection runs per day 0.0",
            "shadow agreement: 1/1 scored predictions agree (100%); 0 unsettled; 0 scored after a bisection "
            "(approximate)",
            "tree-check failures: 0",
            "daily thresholds: ok",
        ]
        for line in expected:
            self.assertIn(line, report.splitlines())
        self.assertIn("T-2991 bar: not met: 1 scored trains", report)


class Stranded(unittest.TestCase):
    """`stranded()` and `--stranded` (T-3344): ready-labelled, not ejected, newest marker past 2 h."""

    @staticmethod
    def routes(now: datetime) -> dict[str, object]:
        def iso(hours_ago: float) -> str:
            return (now - timedelta(hours=hours_ago)).strftime("%Y-%m-%dT%H:%M:%SZ")

        def pull(n: int, *labels: str) -> dict[str, object]:
            return {"number": n, "labels": [{"name": label} for label in labels]}

        def marker(hours_ago: float) -> dict[str, object]:
            return {"created_at": iso(hours_ago), "body": "Marked ready.\n\n<!-- merge-train-ready v1\n-->"}

        ready, ejected = "ready-to-merge", "merge-train-ejected"
        return {
            "pulls": [pull(5, ready), pull(6, ready, ejected), pull(7, ready), pull(8, ready), pull(9)],
            # 5: marked 5 h and 3 h ago (the newest counts); 6: ejected; 7: marked 1 h ago; 8: no marker.
            "issues/5/comments": [marker(5), marker(3)],
            "issues/6/comments": [marker(5)],
            "issues/7/comments": [marker(4), marker(1)],
            "issues/8/comments": [{"created_at": iso(5), "body": "a plain comment"}],
        }

    def test_only_old_markers_on_ready_unejected_prs(self) -> None:
        now = at(10)
        found = stats.Stats(Routed(self.routes(now)), NoGit(), now, now - stats.DAY).stranded()
        self.assertEqual(found, [5])

    def test_the_flag_prints_only_the_stranded_line(self) -> None:
        now = datetime.now(UTC)
        out = io.StringIO()
        with (
            mock.patch.object(stats.gh_client, "from_remote", return_value=Routed(self.routes(now))),
            mock.patch.object(stats, "GitHistory", NoGit),
            contextlib.redirect_stdout(out),
        ):
            self.assertEqual(stats.main(["--stranded"]), 0)
        self.assertEqual(out.getvalue().splitlines(), ["stranded PRs: #5"])


if __name__ == "__main__":
    unittest.main()
